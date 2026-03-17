use std::collections::HashMap;

use caucus_core::MultiProvider;

use crate::types::{GenerationError, NameCandidate};

fn system_prompt(count: usize) -> String {
    format!(
        "You are a creative project naming assistant. Generate exactly {count} unique, memorable project names \
based on the user's description. Output ONLY a numbered list (1-{count}), one name per line, no commentary. \
Names should be lowercase, use only letters, numbers, and hyphens, and be 2-40 characters long. \
Prefer short, catchy, easy-to-spell names that work well as package names and domain names."
    )
}

/// Generate name candidates from multiple LLMs.
pub async fn generate_names(
    provider: &MultiProvider,
    prompt: &str,
    max_names: usize,
) -> (Vec<NameCandidate>, Vec<GenerationError>) {
    let system = system_prompt(max_names);
    let results = provider.generate_all(prompt, Some(&system)).await;

    let mut name_models: HashMap<String, Vec<String>> = HashMap::new();
    let mut errors = Vec::new();

    for (model, result) in results {
        match result {
            Ok(response) => {
                for name in parse_names(&response) {
                    name_models.entry(name).or_default().push(model.clone());
                }
            }
            Err(e) => {
                errors.push(GenerationError {
                    model,
                    error: e.to_string(),
                });
            }
        }
    }

    // Sort by number of models that suggested each name (descending), then alphabetically
    let mut candidates: Vec<NameCandidate> = name_models
        .into_iter()
        .map(|(name, suggested_by)| NameCandidate { name, suggested_by })
        .collect();
    candidates.sort_by(|a, b| {
        b.suggested_by
            .len()
            .cmp(&a.suggested_by.len())
            .then(a.name.cmp(&b.name))
    });
    candidates.truncate(max_names);

    (candidates, errors)
}

/// Parse LLM response into clean name list.
fn parse_names(response: &str) -> Vec<String> {
    response
        .lines()
        .filter_map(|line| {
            let cleaned = line
                .trim()
                .trim_start_matches(|c: char| {
                    c.is_ascii_digit() || c == '.' || c == ')' || c == '-' || c == '*'
                })
                .trim()
                .trim_matches('`')
                .trim_matches('*')
                .trim()
                .to_lowercase();

            if validate_name(&cleaned) {
                Some(cleaned)
            } else {
                None
            }
        })
        .collect()
}

/// Validate a name: lowercase alphanumeric + hyphens, 2-40 chars, no leading/trailing hyphens.
fn validate_name(name: &str) -> bool {
    let len = name.len();
    if !(2..=40).contains(&len) {
        return false;
    }
    if name.starts_with('-') || name.ends_with('-') {
        return false;
    }
    name.chars()
        .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-')
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_names() {
        let response = "1. aurora\n2. drift\n3. nexus\n4. INVALID NAME WITH SPACES\n5. ok-name";
        let names = parse_names(response);
        assert_eq!(names, vec!["aurora", "drift", "nexus", "ok-name"]);
    }

    #[test]
    fn test_parse_names_with_backticks() {
        let response = "1. `aurora`\n2. **drift**\n- nexus";
        let names = parse_names(response);
        assert_eq!(names, vec!["aurora", "drift", "nexus"]);
    }

    #[test]
    fn test_validate_name() {
        assert!(validate_name("aurora"));
        assert!(validate_name("my-app"));
        assert!(validate_name("app2"));
        assert!(!validate_name("a")); // too short
        assert!(!validate_name("-bad"));
        assert!(!validate_name("bad-"));
        assert!(!validate_name("has spaces"));
        assert!(!validate_name("UPPERCASE"));
    }
}
