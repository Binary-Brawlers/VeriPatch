//! Verification pipeline orchestration.

use std::path::PathBuf;

use anyhow::Result;
use veripatch_rules::rule::{
    Assumption, RiskSeverity, RuleFinding, RuleInputLine, analyze_lines, detect_assumptions,
};
use veripatch_runners::runner::{CheckResult, CheckStatus, RunnerContext, run_default_checks};

use crate::{
    diff::{DiffLineKind, ParsedDiff, parse_unified_diff},
    verdict::{Verdict, VerificationResult},
};

#[derive(Debug, Clone)]
pub struct VerificationInput {
    pub repo_path: PathBuf,
    pub diff_text: String,
}

pub async fn verify(input: VerificationInput) -> Result<VerificationResult> {
    let parsed_diff = parse_unified_diff(&input.diff_text)?;
    let rule_lines = collect_added_lines(&parsed_diff);
    let checks = run_default_checks(&RunnerContext {
        repo_path: input.repo_path.clone(),
    })
    .await?;
    let risky_patterns = analyze_lines(&rule_lines);
    let assumptions = detect_assumptions(&rule_lines);
    let warnings = build_warnings(&parsed_diff, &checks, &risky_patterns);
    let score = calculate_score(&checks, &risky_patterns, &assumptions, &warnings);
    let verdict = determine_verdict(&checks, &risky_patterns, score);

    Ok(VerificationResult {
        repo_path: input.repo_path,
        diff: parsed_diff,
        verdict,
        score,
        checks,
        warnings,
        assumptions,
        risky_patterns,
    })
}

fn collect_added_lines(parsed_diff: &ParsedDiff) -> Vec<RuleInputLine> {
    let mut lines = Vec::new();

    for file in &parsed_diff.files {
        let file_path = file.display_path();

        for hunk in &file.hunks {
            for line in &hunk.lines {
                if line.kind == DiffLineKind::Addition {
                    lines.push(RuleInputLine {
                        file_path: file_path.clone(),
                        line_number: line.new_line_number,
                        content: line.content.clone(),
                    });
                }
            }
        }
    }

    lines
}

fn build_warnings(
    parsed_diff: &ParsedDiff,
    checks: &[CheckResult],
    risky_patterns: &[RuleFinding],
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

    warnings
}

fn calculate_score(
    checks: &[CheckResult],
    risky_patterns: &[RuleFinding],
    assumptions: &[Assumption],
    warnings: &[String],
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
    score = score.saturating_add((warnings.len().min(3) as u8) * 4);
    score.min(100)
}

fn determine_verdict(checks: &[CheckResult], risky_patterns: &[RuleFinding], score: u8) -> Verdict {
    if checks
        .iter()
        .any(|check| check.status == CheckStatus::Failed && check.name.contains("Compile"))
    {
        Verdict::Broken
    } else if risky_patterns
        .iter()
        .any(|finding| finding.severity == RiskSeverity::High)
    {
        Verdict::Risky
    } else if checks
        .iter()
        .any(|check| check.status == CheckStatus::Failed)
        || score >= 35
    {
        Verdict::Risky
    } else {
        Verdict::Safe
    }
}
