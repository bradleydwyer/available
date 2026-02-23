use clap::{Parser, Subcommand};

use available::check;
use available::generate;
use available::mcp::AvailableMcp;
use available::provider;
use available::types::{AvailableResult, Config, NameResult};
use rmcp::{ServiceExt, transport::stdio};

#[derive(Parser)]
#[command(
    name = "available",
    about = "AI-powered project name finder — generates names and checks availability",
    version
)]
struct Cli {
    #[command(subcommand)]
    command: Option<Command>,

    /// Project description for AI name generation, or names to check with --check
    prompt: Vec<String>,

    /// Check specific names instead of generating (comma-separated)
    #[arg(long)]
    check: Option<String>,

    /// Comma-separated model names (default: auto-detect from API keys)
    #[arg(long)]
    models: Option<String>,

    /// Comma-separated TLDs to check (default: com,dev,io)
    #[arg(long)]
    tlds: Option<String>,

    /// Comma-separated registry IDs (default: popular registries)
    #[arg(long)]
    registries: Option<String>,

    /// Maximum number of names to generate (default: 20)
    #[arg(long, default_value = "20")]
    max_names: usize,

    /// Output results as JSON
    #[arg(long)]
    json: bool,

    /// Show per-registry and per-domain detail
    #[arg(short, long)]
    verbose: bool,
}

#[derive(Subcommand)]
enum Command {
    /// Start MCP server (stdio transport)
    Mcp,
}

fn build_config(cli: &Cli) -> Config {
    let mut config = Config::default();
    if let Some(ref tlds) = cli.tlds {
        config.tlds = tlds.split(',').map(|s| s.trim().to_string()).collect();
    }
    if let Some(ref registries) = cli.registries {
        config.registry_ids = registries.split(',').map(|s| s.trim().to_string()).collect();
    }
    config.max_names = cli.max_names;
    config
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    // MCP server mode
    if let Some(Command::Mcp) = cli.command {
        let server = AvailableMcp::new();
        let service = server.serve(stdio()).await?;
        service.waiting().await?;
        return Ok(());
    }

    let config = build_config(&cli);

    // Check-only mode
    if let Some(ref names_str) = cli.check {
        let names: Vec<String> = names_str.split(',').map(|s| s.trim().to_string()).collect();
        let results = check::check_name_strings(&names, &config).await;
        let output = AvailableResult {
            results,
            models_used: vec![],
            errors: vec![],
        };
        if cli.json {
            println!("{}", serde_json::to_string_pretty(&output)?);
        } else {
            print_results(&output.results, cli.verbose);
        }
        return Ok(());
    }

    // Generation mode
    let prompt = cli.prompt.join(" ");
    if prompt.is_empty() {
        eprintln!("Usage: available \"project description\"");
        eprintln!("       available --check name1,name2,name3");
        eprintln!("       available mcp");
        eprintln!();
        eprintln!("Run 'available --help' for more information.");
        std::process::exit(1);
    }

    let models = match cli.models {
        Some(ref m) => m.split(',').map(|s| s.trim().to_string()).collect(),
        None => provider::default_models(),
    };
    if models.is_empty() {
        eprintln!("No API keys found. Set at least one of: ANTHROPIC_API_KEY, OPENAI_API_KEY, GOOGLE_API_KEY, XAI_API_KEY");
        std::process::exit(1);
    }

    let multi = provider::build_provider(&models)?;
    eprintln!("Generating names with: {}", models.join(", "));

    let (candidates, errors) = generate::generate_names(&multi, &prompt, config.max_names).await;

    for error in &errors {
        eprintln!("Warning: {} failed: {}", error.model, error.error);
    }

    if candidates.is_empty() {
        eprintln!("No valid names generated. Try a different prompt.");
        std::process::exit(1);
    }

    eprintln!("Checking {} names...", candidates.len());
    let results = check::check_names(&candidates, &config).await;

    let output = AvailableResult {
        results,
        models_used: models,
        errors,
    };

    if cli.json {
        println!("{}", serde_json::to_string_pretty(&output)?);
    } else {
        print_results(&output.results, cli.verbose);
    }

    Ok(())
}

fn print_results(results: &[NameResult], verbose: bool) {
    for result in results {
        let bar = score_bar(result.score);
        let com_status = domain_status(&result.domains.details, "com");
        let dev_status = domain_status(&result.domains.details, "dev");
        let io_status = domain_status(&result.domains.details, "io");

        println!(
            "  {bar} {score:.0}%  {name:<20} .com{com} .dev{dev} .io{io}  pkg: {pkg_avail}/{pkg_total} available",
            bar = bar,
            score = result.score * 100.0,
            name = result.name,
            com = com_status,
            dev = dev_status,
            io = io_status,
            pkg_avail = result.packages.available,
            pkg_total = result.packages.total,
        );

        if verbose {
            if !result.suggested_by.is_empty() {
                println!("         suggested by: {}", result.suggested_by.join(", "));
            }
            for d in &result.domains.details {
                let symbol = availability_symbol(&d.available);
                println!("         {symbol} {:<24} {}", d.domain, d.available);
            }
            for p in &result.packages.details {
                let symbol = availability_symbol(&p.available);
                println!("         {symbol} {:<24} {}", p.registry, p.available);
            }
            println!();
        }
    }
}

fn score_bar(score: f64) -> String {
    let filled = (score * 10.0).round() as usize;
    let empty = 10 - filled;
    format!("[{}{}]", "#".repeat(filled), "-".repeat(empty))
}

fn domain_status(details: &[available::types::DomainDetail], tld: &str) -> &'static str {
    for d in details {
        if d.domain.ends_with(&format!(".{tld}")) {
            return match d.available.as_str() {
                "available" => "[+]",
                "registered" => "[-]",
                _ => "[?]",
            };
        }
    }
    "   "
}

fn availability_symbol(status: &str) -> &'static str {
    match status {
        "available" => "[+]",
        "taken" | "registered" => "[-]",
        _ => "[?]",
    }
}
