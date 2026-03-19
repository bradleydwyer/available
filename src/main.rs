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
    about = "Check project name availability across domains and registries",
    version
)]
struct Cli {
    #[command(subcommand)]
    command: Option<Command>,

    /// Names to check (space or comma-separated), or description when using --generate
    prompt: Vec<String>,

    /// Generate names from a description instead of checking
    #[arg(long)]
    generate: bool,

    /// Comma-separated model names (default: auto-detect from API keys)
    #[arg(long)]
    models: Option<String>,

    /// Comma-separated TLDs to check (default: com,dev,io,app)
    #[arg(long)]
    tlds: Option<String>,

    /// Comma-separated registry IDs (default: popular registries)
    #[arg(long)]
    registries: Option<String>,

    /// Comma-separated app store IDs to check (default: app_store, google_play)
    #[arg(long)]
    stores: Option<String>,

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
        config.registry_ids = registries
            .split(',')
            .map(|s| s.trim().to_string())
            .collect();
    }
    if let Some(ref stores) = cli.stores {
        config.store_ids = stores.split(',').map(|s| s.trim().to_string()).collect();
    }
    config.max_names = cli.max_names;
    config
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenvy::dotenv().ok();
    let cli = Cli::parse();

    // MCP server mode
    if let Some(Command::Mcp) = cli.command {
        let server = AvailableMcp::new();
        let service = server.serve(stdio()).await?;
        service.waiting().await?;
        return Ok(());
    }

    let config = build_config(&cli);

    let input = cli.prompt.join(" ");
    if input.is_empty() {
        eprintln!("Usage: available name1 name2 name3");
        eprintln!("       available --generate \"project description\"");
        eprintln!("       available mcp");
        eprintln!();
        eprintln!("Run 'available --help' for more information.");
        std::process::exit(1);
    }

    // Generation mode (opt-in with --generate)
    if cli.generate {
        let models = match cli.models {
            Some(ref m) => m.split(',').map(|s| s.trim().to_string()).collect(),
            None => provider::default_models(),
        };
        if models.is_empty() {
            eprintln!(
                "No API keys found. Set at least one of: ANTHROPIC_API_KEY, OPENAI_API_KEY, GOOGLE_API_KEY, XAI_API_KEY"
            );
            std::process::exit(1);
        }

        let multi = provider::build_provider(&models)?;
        eprintln!("Generating names with: {}", models.join(", "));

        let (candidates, errors) = generate::generate_names(&multi, &input, config.max_names).await;

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

        return Ok(());
    }

    // Default: check mode — treat positional args as names
    let names: Vec<String> = input
        .split([' ', ','])
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect();
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

    Ok(())
}

fn print_results(results: &[NameResult], verbose: bool) {
    for result in results {
        let bar = score_bar(result.score);
        let com_status = domain_status(&result.domains.details, "com");
        let dev_status = domain_status(&result.domains.details, "dev");
        let io_status = domain_status(&result.domains.details, "io");
        let app_status = domain_status(&result.domains.details, "app");

        let store_info = if result.stores.total > 0 {
            format!(
                "  stores: {}/{}",
                result.stores.available, result.stores.total
            )
        } else {
            String::new()
        };

        println!(
            "  {bar} {score:.0}%  {name:<20} .com{com} .dev{dev} .io{io} .app{app}  pkg: {pkg_avail}/{pkg_total} available{stores}",
            bar = bar,
            score = result.score * 100.0,
            name = result.name,
            com = com_status,
            dev = dev_status,
            io = io_status,
            app = app_status,
            pkg_avail = result.packages.available,
            pkg_total = result.packages.total,
            stores = store_info,
        );

        if verbose {
            if !result.suggested_by.is_empty() {
                println!("         suggested by: {}", result.suggested_by.join(", "));
            }
            for d in &result.domains.details {
                let symbol = availability_symbol(&d.available);
                let site_info = d
                    .site
                    .as_deref()
                    .map(|s| format!(" ({s})"))
                    .unwrap_or_default();
                println!(
                    "         {symbol} {:<24} {}{}",
                    d.domain, d.available, site_info
                );
            }
            for p in &result.packages.details {
                let symbol = availability_symbol(&p.available);
                println!("         {symbol} {:<24} {}", p.registry, p.available);
            }
            for s in &result.stores.details {
                let symbol = availability_symbol(&s.available);
                println!(
                    "         {symbol} {:<24} {} ({} similar)",
                    s.store, s.available, s.similar_count
                );
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

fn domain_status(details: &[available::types::DomainDetail], tld: &str) -> String {
    for d in details {
        if d.domain.ends_with(&format!(".{tld}")) {
            return match d.available.as_str() {
                "available" => "[+]".into(),
                "registered" => match d.site.as_deref() {
                    Some("parked") => "[-P]".into(),
                    Some("active") => "[-A]".into(),
                    Some("redirect") => "[-R]".into(),
                    Some("unreachable") => "[-X]".into(),
                    _ => "[-]".into(),
                },
                _ => "[?]".into(),
            };
        }
    }
    "   ".into()
}

fn availability_symbol(status: &str) -> &'static str {
    match status {
        "available" => "[+]",
        "taken" | "registered" => "[-]",
        _ => "[?]",
    }
}
