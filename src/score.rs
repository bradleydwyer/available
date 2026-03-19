use crate::types::{
    DomainDetail, DomainSummary, NameResult, PackageDetail, PackageSummary, StoreDetail,
    StoreSummary,
};

/// Score a name result based on domain, package, and store availability.
///
/// Weight distribution (adjusts dynamically based on what's checked):
/// - Domains: 50% (without stores) or 40% (with stores)
///   - .com gets half the domain budget, rest split evenly
/// - Package registries: 50% (without stores) or 40% (with stores)
/// - App stores: 20% (when present)
pub fn score(result: &NameResult) -> f64 {
    let has_stores = result.stores.total > 0;
    let domain_score = score_domains(&result.domains, has_stores);
    let package_score = score_packages(&result.packages, has_stores);
    let store_score = score_stores(&result.stores);
    let raw = domain_score + package_score + store_score;

    (raw * 10000.0).round() / 10000.0
}

fn score_domains(summary: &DomainSummary, has_stores: bool) -> f64 {
    if summary.details.is_empty() {
        return 0.0;
    }
    let total_weight = if has_stores { 0.40 } else { 0.50 };
    let com_weight = total_weight / 2.0;
    let other_count = summary.details.len().saturating_sub(1).max(1);
    let other_weight = (total_weight - com_weight) / other_count as f64;

    let mut total = 0.0;
    for detail in &summary.details {
        let weight = if domain_tld(&detail.domain) == "com" {
            com_weight
        } else {
            other_weight
        };
        total += weight * availability_score(&detail.available);
    }
    total
}

fn score_packages(summary: &PackageSummary, has_stores: bool) -> f64 {
    if summary.details.is_empty() {
        return 0.0;
    }
    let total_weight = if has_stores { 0.40 } else { 0.50 };
    let weight_per_registry = total_weight / summary.details.len() as f64;
    summary
        .details
        .iter()
        .map(|d| weight_per_registry * availability_score(&d.available))
        .sum()
}

fn score_stores(summary: &StoreSummary) -> f64 {
    if summary.details.is_empty() {
        return 0.0;
    }
    let weight_per_store = 0.20 / summary.details.len() as f64;
    summary
        .details
        .iter()
        .map(|d| weight_per_store * availability_score(&d.available))
        .sum()
}

fn availability_score(status: &str) -> f64 {
    match status {
        "available" => 1.0,
        "unknown" => 0.5,
        _ => 0.0, // taken, registered
    }
}

fn domain_tld(domain: &str) -> &str {
    domain.rsplit('.').next().unwrap_or("")
}

/// Build a DomainSummary from domain-check results.
pub fn build_domain_summary(results: &[parked::types::DomainResult]) -> DomainSummary {
    let details: Vec<DomainDetail> = results
        .iter()
        .map(|r| DomainDetail {
            domain: r.domain.clone(),
            available: format!("{}", r.available).to_lowercase(),
            site: r
                .site
                .as_ref()
                .map(|s| format!("{}", s.classification).to_lowercase()),
        })
        .collect();

    let available = details
        .iter()
        .filter(|d| d.available == "available")
        .count();
    let registered = details
        .iter()
        .filter(|d| d.available == "registered")
        .count();
    let unknown = details.iter().filter(|d| d.available == "unknown").count();

    DomainSummary {
        available,
        registered,
        unknown,
        total: details.len(),
        details,
    }
}

/// Build a StoreSummary from app-store-check results.
pub fn build_store_summary(result: &published::types::CheckResult) -> StoreSummary {
    let details: Vec<StoreDetail> = result
        .results
        .iter()
        .map(|r| StoreDetail {
            store: r.store_name.clone(),
            available: format!("{}", r.available).to_lowercase(),
            similar_count: r.similar_count,
        })
        .collect();

    StoreSummary {
        available: result.summary.available,
        taken: result.summary.taken,
        unknown: result.summary.unknown,
        total: result.summary.total,
        details,
    }
}

/// Build a PackageSummary from pkg-check results.
pub fn build_package_summary(result: &staked::types::CheckResult) -> PackageSummary {
    let details: Vec<PackageDetail> = result
        .results
        .iter()
        .map(|r| PackageDetail {
            registry: r.registry_name.clone(),
            available: format!("{}", r.available).to_lowercase(),
        })
        .collect();

    PackageSummary {
        available: result.summary.available,
        taken: result.summary.taken,
        unknown: result.summary.unknown,
        total: result.summary.total,
        details,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_perfect_score() {
        let result = NameResult {
            name: "test".into(),
            score: 0.0,
            suggested_by: vec![],
            domains: DomainSummary {
                available: 4,
                registered: 0,
                unknown: 0,
                total: 4,
                details: vec![
                    DomainDetail {
                        domain: "test.com".into(),
                        available: "available".into(),
                        site: None,
                    },
                    DomainDetail {
                        domain: "test.dev".into(),
                        available: "available".into(),
                        site: None,
                    },
                    DomainDetail {
                        domain: "test.io".into(),
                        available: "available".into(),
                        site: None,
                    },
                    DomainDetail {
                        domain: "test.app".into(),
                        available: "available".into(),
                        site: None,
                    },
                ],
            },
            packages: PackageSummary {
                available: 2,
                taken: 0,
                unknown: 0,
                total: 2,
                details: vec![
                    PackageDetail {
                        registry: "npm".into(),
                        available: "available".into(),
                    },
                    PackageDetail {
                        registry: "crates".into(),
                        available: "available".into(),
                    },
                ],
            },
            stores: StoreSummary {
                available: 0,
                taken: 0,
                unknown: 0,
                total: 0,
                details: vec![],
            },
        };
        let s = score(&result);
        assert!((s - 1.0).abs() < 0.001);
    }

    #[test]
    fn test_all_unknown_score() {
        // Regression test: all-unknown inputs should yield exactly 0.5 after rounding,
        // not 0.49999999999999994 due to IEEE 754 drift.
        let result = NameResult {
            name: "test".into(),
            score: 0.0,
            suggested_by: vec![],
            domains: DomainSummary {
                available: 0,
                registered: 0,
                unknown: 4,
                total: 4,
                details: vec![
                    DomainDetail {
                        domain: "test.com".into(),
                        available: "unknown".into(),
                        site: None,
                    },
                    DomainDetail {
                        domain: "test.dev".into(),
                        available: "unknown".into(),
                        site: None,
                    },
                    DomainDetail {
                        domain: "test.io".into(),
                        available: "unknown".into(),
                        site: None,
                    },
                    DomainDetail {
                        domain: "test.app".into(),
                        available: "unknown".into(),
                        site: None,
                    },
                ],
            },
            packages: PackageSummary {
                available: 0,
                taken: 0,
                unknown: 2,
                total: 2,
                details: vec![
                    PackageDetail {
                        registry: "npm".into(),
                        available: "unknown".into(),
                    },
                    PackageDetail {
                        registry: "crates".into(),
                        available: "unknown".into(),
                    },
                ],
            },
            stores: StoreSummary {
                available: 0,
                taken: 0,
                unknown: 0,
                total: 0,
                details: vec![],
            },
        };
        assert_eq!(score(&result), 0.5);
    }

    #[test]
    fn test_zero_score() {
        let result = NameResult {
            name: "test".into(),
            score: 0.0,
            suggested_by: vec![],
            domains: DomainSummary {
                available: 0,
                registered: 4,
                unknown: 0,
                total: 4,
                details: vec![
                    DomainDetail {
                        domain: "test.com".into(),
                        available: "registered".into(),
                        site: None,
                    },
                    DomainDetail {
                        domain: "test.dev".into(),
                        available: "registered".into(),
                        site: None,
                    },
                    DomainDetail {
                        domain: "test.io".into(),
                        available: "registered".into(),
                        site: None,
                    },
                    DomainDetail {
                        domain: "test.app".into(),
                        available: "registered".into(),
                        site: None,
                    },
                ],
            },
            packages: PackageSummary {
                available: 0,
                taken: 2,
                unknown: 0,
                total: 2,
                details: vec![
                    PackageDetail {
                        registry: "npm".into(),
                        available: "taken".into(),
                    },
                    PackageDetail {
                        registry: "crates".into(),
                        available: "taken".into(),
                    },
                ],
            },
            stores: StoreSummary {
                available: 0,
                taken: 0,
                unknown: 0,
                total: 0,
                details: vec![],
            },
        };
        let s = score(&result);
        assert!((s - 0.0).abs() < 0.001);
    }
}
