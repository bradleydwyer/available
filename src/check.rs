use std::sync::Arc;
use tokio::sync::Semaphore;

use crate::score::{build_domain_summary, build_package_summary, score};
use crate::types::{Config, NameCandidate, NameResult};

/// Check availability of name candidates across domains and package registries.
pub async fn check_names(candidates: &[NameCandidate], config: &Config) -> Vec<NameResult> {
    let semaphore = Arc::new(Semaphore::new(5));
    let mut handles = Vec::new();

    for candidate in candidates {
        let candidate = candidate.clone();
        let config = config.clone();
        let sem = Arc::clone(&semaphore);
        handles.push(tokio::spawn(async move {
            let _permit = sem.acquire().await.unwrap();
            check_single(&candidate, &config).await
        }));
    }

    let mut results = Vec::new();
    for handle in handles {
        if let Ok(result) = handle.await {
            results.push(result);
        }
    }

    results.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal));
    results
}

async fn check_single(candidate: &NameCandidate, config: &Config) -> NameResult {
    let domains: Vec<String> = config
        .tlds
        .iter()
        .map(|tld| format!("{}.{}", candidate.name, tld))
        .collect();

    let registries = resolve_registries(&config.registry_ids);

    let (domain_results, pkg_result) = tokio::join!(
        domain_check::checker::check_domains(&domains),
        pkg_check::checker::check_package(&candidate.name, &registries),
    );

    let domain_summary = build_domain_summary(&domain_results);
    let package_summary = build_package_summary(&pkg_result);

    let mut result = NameResult {
        name: candidate.name.clone(),
        score: 0.0,
        suggested_by: candidate.suggested_by.clone(),
        domains: domain_summary,
        packages: package_summary,
    };
    result.score = score(&result);
    result
}

fn resolve_registries(ids: &[String]) -> Vec<&'static pkg_check::registry::Registry> {
    if ids.is_empty() {
        pkg_check::registry::popular_registries()
    } else {
        pkg_check::registry::registries_by_ids(ids)
    }
}

/// Check specific names (without LLM generation) — used by --check mode.
pub async fn check_name_strings(names: &[String], config: &Config) -> Vec<NameResult> {
    let candidates: Vec<NameCandidate> = names
        .iter()
        .map(|name| NameCandidate {
            name: name.clone(),
            suggested_by: vec![],
        })
        .collect();
    check_names(&candidates, config).await
}
