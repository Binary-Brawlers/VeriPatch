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
    let changed_paths = parsed_diff.changed_paths();
    let checks = run_default_checks(
        &RunnerContext {
            repo_path: input.repo_path.clone(),
        },
        &changed_paths,
    )
    .await?;
    let risky_patterns = analyze_lines(&rule_lines);
    let assumptions = detect_assumptions(&rule_lines);
    let dependency_notes = detect_dependency_notes(&parsed_diff);
    let warnings = build_warnings(&parsed_diff, &checks, &risky_patterns, &dependency_notes);
    let score = calculate_score(
        &checks,
        &risky_patterns,
        &assumptions,
        &warnings,
        &dependency_notes,
    );
    let verdict = determine_verdict(&checks, &risky_patterns, score);

    Ok(VerificationResult {
        repo_path: input.repo_path,
        diff: parsed_diff,
        verdict,
        score,
        checks,
        warnings,
        assumptions,
        dependency_notes,
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

fn calculate_score(
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

fn detect_dependency_notes(parsed_diff: &ParsedDiff) -> Vec<String> {
    let mut notes = Vec::new();
    let cargo_manifest_files: Vec<_> = parsed_diff
        .files
        .iter()
        .filter(|file| file.display_path().ends_with("Cargo.toml"))
        .collect();
    let has_cargo_lock = parsed_diff
        .files
        .iter()
        .any(|file| file.display_path() == "Cargo.lock");

    if has_cargo_lock && cargo_manifest_files.is_empty() {
        notes.push(
            "`Cargo.lock` changed without a direct `Cargo.toml` manifest change.".to_string(),
        );
    }

    for file in cargo_manifest_files {
        let mut current_section: Option<String> = None;

        for hunk in &file.hunks {
            for line in &hunk.lines {
                if matches!(line.kind, DiffLineKind::Context | DiffLineKind::Addition) {
                    if let Some(section) = parse_toml_section(&line.content) {
                        current_section = Some(section.to_string());
                        continue;
                    }
                }

                if line.kind == DiffLineKind::Addition
                    && current_section
                        .as_deref()
                        .is_some_and(is_dependency_section)
                    && let Some(dependency_name) = parse_dependency_entry(&line.content)
                {
                    notes.push(format!(
                        "New Rust dependency added in `{}`: `{}`.",
                        file.display_path(),
                        dependency_name
                    ));
                }
            }
        }
    }

    notes
}

fn parse_toml_section(line: &str) -> Option<&str> {
    let trimmed = line.trim();
    trimmed.strip_prefix('[')?.strip_suffix(']')
}

fn is_dependency_section(section: &str) -> bool {
    matches!(
        section,
        "dependencies" | "dev-dependencies" | "build-dependencies" | "workspace.dependencies"
    )
}

fn parse_dependency_entry(line: &str) -> Option<&str> {
    let trimmed = line.trim();
    if trimmed.starts_with('#') || !trimmed.contains('=') {
        return None;
    }

    trimmed
        .split('=')
        .next()
        .map(str::trim)
        .filter(|name| !name.is_empty())
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

#[cfg(test)]
mod tests {
    use super::detect_dependency_notes;
    use crate::diff::{ChangedFile, DiffHunk, DiffLine, DiffLineKind, FileChangeType, ParsedDiff};

    #[test]
    fn reports_added_cargo_dependencies() {
        let parsed_diff = ParsedDiff {
            files: vec![ChangedFile {
                old_path: Some("Cargo.toml".to_string()),
                new_path: Some("Cargo.toml".to_string()),
                change_type: FileChangeType::Modified,
                additions: 2,
                deletions: 0,
                hunks: vec![DiffHunk {
                    header: "@@ -1,2 +1,4 @@".to_string(),
                    lines: vec![
                        DiffLine {
                            kind: DiffLineKind::Context,
                            content: "[dependencies]".to_string(),
                            old_line_number: Some(1),
                            new_line_number: Some(1),
                        },
                        DiffLine {
                            kind: DiffLineKind::Addition,
                            content: "serde = \"1\"".to_string(),
                            old_line_number: None,
                            new_line_number: Some(2),
                        },
                    ],
                }],
            }],
            total_additions: 1,
            total_deletions: 0,
        };

        let notes = detect_dependency_notes(&parsed_diff);
        assert_eq!(
            notes,
            vec!["New Rust dependency added in `Cargo.toml`: `serde`.".to_string()]
        );
    }
}
