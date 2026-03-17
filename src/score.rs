use crate::types::{DomainDetail, DomainSummary, NameResult, PackageDetail, PackageSummary};

/// Score a name result based on domain and package availability.
///
/// Weights:
/// - .com domain: 30%
/// - .dev domain: 10%
/// - .io domain: 10%
/// - Package registries: 50% (split evenly)
pub fn score(result: &NameResult) -> f64 {
    let domain_score = score_domains(&result.domains);
    let package_score = score_packages(&result.packages);
    let raw = domain_score + package_score;

    (raw * 10000.0).round() / 10000.0
}

fn score_domains(summary: &DomainSummary) -> f64 {
    let mut total = 0.0;
    for detail in &summary.details {
        let weight = match domain_tld(&detail.domain) {
            "com" => 0.30,
            "dev" => 0.10,
            "io" => 0.10,
            _ => 0.0,
        };
        total += weight * availability_score(&detail.available);
    }
    total
}

fn score_packages(summary: &PackageSummary) -> f64 {
    if summary.details.is_empty() {
        return 0.0;
    }
    let weight_per_registry = 0.50 / summary.details.len() as f64;
    summary
        .details
        .iter()
        .map(|d| weight_per_registry * availability_score(&d.available))
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
                available: 3,
                registered: 0,
                unknown: 0,
                total: 3,
                details: vec![
                    DomainDetail {
                        domain: "test.com".into(),
                        available: "available".into(),
                    },
                    DomainDetail {
                        domain: "test.dev".into(),
                        available: "available".into(),
                    },
                    DomainDetail {
                        domain: "test.io".into(),
                        available: "available".into(),
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
                unknown: 3,
                total: 3,
                details: vec![
                    DomainDetail {
                        domain: "test.com".into(),
                        available: "unknown".into(),
                    },
                    DomainDetail {
                        domain: "test.dev".into(),
                        available: "unknown".into(),
                    },
                    DomainDetail {
                        domain: "test.io".into(),
                        available: "unknown".into(),
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
                registered: 3,
                unknown: 0,
                total: 3,
                details: vec![
                    DomainDetail {
                        domain: "test.com".into(),
                        available: "registered".into(),
                    },
                    DomainDetail {
                        domain: "test.dev".into(),
                        available: "registered".into(),
                    },
                    DomainDetail {
                        domain: "test.io".into(),
                        available: "registered".into(),
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
        };
        let s = score(&result);
        assert!((s - 0.0).abs() < 0.001);
    }
}
