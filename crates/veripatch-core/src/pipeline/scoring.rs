use veripatch_rules::rule::{Assumption, RiskSeverity, RuleFinding};
use veripatch_runners::runner::{CheckResult, CheckStatus};

use crate::{diff::ParsedDiff, verdict::Verdict};

pub(super) fn build_warnings(
    parsed_diff: &ParsedDiff,
    checks: &[CheckResult],
    risky_patterns: &[RuleFinding],
    dependency_notes: &[String],
) -> Vec<String> {
    let mut warnings = Vec::new();

    let skipped_checks = checks
        .iter()
        .filter(|check| check.status == CheckStatus::Skipped)
        .count();
    if skipped_checks > 0 {
        warnings.push(format!(
            "{skipped_checks} verification check(s) were skipped because the project type or local tooling was not available."
        ));
    }

    if parsed_diff.files.len() > 10 {
        warnings.push(format!(
            "This change touches {} files, which increases review risk.",
            parsed_diff.files.len()
        ));
    }

    if risky_patterns
        .iter()
        .any(|finding| finding.severity == RiskSeverity::High)
    {
        warnings.push("High-severity risky patterns were detected in added lines.".to_string());
    }

    if !dependency_notes.is_empty() {
        warnings.push(
            "Dependency-related changes were detected and should receive extra review.".to_string(),
        );
    }

    warnings
}

pub(super) fn calculate_score(
    checks: &[CheckResult],
    risky_patterns: &[RuleFinding],
    assumptions: &[Assumption],
    warnings: &[String],
    dependency_notes: &[String],
) -> u8 {
    let mut score = 0u8;

    for check in checks {
        score = score.saturating_add(match check.status {
            CheckStatus::Passed => 0,
            CheckStatus::Skipped => 8,
            CheckStatus::Failed if check.name.contains("Compile") => 70,
            CheckStatus::Failed => 25,
        });
    }

    for finding in risky_patterns {
        score = score.saturating_add(match finding.severity {
            RiskSeverity::Low => 6,
            RiskSeverity::Medium => 14,
            RiskSeverity::High => 24,
        });
    }

    score = score.saturating_add((assumptions.len().min(4) as u8) * 5);
    score = score.saturating_add((dependency_notes.len().min(3) as u8) * 6);
    score = score.saturating_add((warnings.len().min(3) as u8) * 4);
    score.min(100)
}

pub(super) fn determine_verdict(
    checks: &[CheckResult],
    risky_patterns: &[RuleFinding],
    score: u8,
) -> Verdict {
    if checks
        .iter()
        .any(|check| check.status == CheckStatus::Failed && check.name.contains("Compile"))
    {
        Verdict::Broken
    } else if risky_patterns
        .iter()
        .any(|finding| finding.severity == RiskSeverity::High)
        || checks
            .iter()
            .any(|check| check.status == CheckStatus::Failed)
        || score >= 35
    {
        Verdict::Risky
    } else {
        Verdict::Safe
    }
}
