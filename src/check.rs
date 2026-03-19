use std::sync::Arc;
use tokio::sync::Semaphore;

use crate::score::{build_domain_summary, build_package_summary, build_store_summary, score};
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

    results.sort_by(|a, b| {
        b.score
            .partial_cmp(&a.score)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    results
}

async fn check_single(candidate: &NameCandidate, config: &Config) -> NameResult {
    let domains: Vec<String> = config
        .tlds
        .iter()
        .map(|tld| format!("{}.{}", candidate.name, tld))
        .collect();

    let registries = resolve_registries(config);
    let stores = resolve_stores(&config.store_ids);

    let parked_opts = parked::checker::CheckOptions::default();
    let (domain_results, pkg_result, store_result) = tokio::join!(
        parked::checker::check_domains(&domains, &parked_opts),
        staked::checker::check_package(&candidate.name, &registries),
        published::checker::check_app(&candidate.name, &stores),
    );

    let domain_summary = build_domain_summary(&domain_results);
    let package_summary = build_package_summary(&pkg_result);
    let store_summary = build_store_summary(&store_result);

    let mut result = NameResult {
        name: candidate.name.clone(),
        score: 0.0,
        suggested_by: candidate.suggested_by.clone(),
        domains: domain_summary,
        packages: package_summary,
        stores: store_summary,
    };
    result.score = score(&result);
    result
}

fn resolve_registries(config: &Config) -> Vec<&'static staked::registry::Registry> {
    if !config.languages.is_empty() {
        staked::registry::registries_by_languages(&config.languages)
    } else if config.all_registries {
        staked::registry::all_registries().iter().collect()
    } else if config.registry_ids.is_empty() {
        staked::registry::popular_registries()
    } else {
        staked::registry::registries_by_ids(&config.registry_ids)
    }
}

fn resolve_stores(ids: &[String]) -> Vec<published::store::Store> {
    if ids.is_empty() {
        published::store::all_stores().to_vec()
    } else {
        published::store::stores_by_ids(ids)
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
