//! Verification pipeline orchestration.

use std::path::PathBuf;

use anyhow::Result;
use serde::Deserialize;
use tokio::process::Command;
use veripatch_rules::rule::{
    Assumption, RiskSeverity, RuleFinding, RuleInputLine, analyze_lines, detect_assumptions,
};
use veripatch_runners::runner::{CheckResult, CheckStatus, RunnerContext, run_default_checks};

use crate::{
    diff::{DiffLineKind, ParsedDiff, parse_unified_diff},
    repo::{VerificationMode, prepare_repository},
    verdict::{Verdict, VerificationResult},
};

#[derive(Debug, Clone)]
pub struct VerificationInput {
    pub repo_path: PathBuf,
    pub diff_text: String,
    pub mode: VerificationMode,
}

pub async fn verify(input: VerificationInput) -> Result<VerificationResult> {
    let parsed_diff = parse_unified_diff(&input.diff_text)?;
    let prepared_repo = prepare_repository(&input.repo_path, &input.diff_text, input.mode).await?;
    let rule_lines = collect_added_lines(&parsed_diff);
    let changed_paths = parsed_diff.changed_paths();
    let checks = run_default_checks(
        &RunnerContext {
            repo_path: prepared_repo.execution_path().to_path_buf(),
        },
        &changed_paths,
    )
    .await?;
    let risky_patterns = analyze_lines(&rule_lines);
    let assumptions = detect_assumptions(&rule_lines);
    let dependency_notes =
        detect_dependency_notes(&parsed_diff, prepared_repo.execution_path()).await?;
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

async fn detect_dependency_notes(
    parsed_diff: &ParsedDiff,
    execution_repo_path: &std::path::Path,
) -> Result<Vec<String>> {
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

    let dependency_changes = cargo_manifest_files
        .iter()
        .flat_map(|file| extract_dependency_changes(file))
        .collect::<Vec<_>>();

    let metadata_packages = if dependency_changes.is_empty() {
        Vec::new()
    } else {
        load_cargo_metadata_packages(execution_repo_path)
            .await
            .unwrap_or_default()
    };

    for change in dependency_changes {
        notes.push(format!(
            "New Rust dependency added in `{}`: `{}`.",
            change.manifest_path, change.name
        ));

        if change.spec.contains("git =") {
            notes.push(format!(
                "Dependency `{}` uses a git source and should be reviewed for provenance and pinning.",
                change.name
            ));
        }

        if change.spec.contains("path =") {
            notes.push(format!(
                "Dependency `{}` uses a local path source and may rely on repository-specific layout assumptions.",
                change.name
            ));
        }

        if change.spec.contains('"') && change.spec.contains("*") {
            notes.push(format!(
                "Dependency `{}` appears to use a wildcard version requirement.",
                change.name
            ));
        }

        if let Some(package) = metadata_packages
            .iter()
            .find(|package| package.name == change.name)
        {
            if let Some(license) = &package.license {
                notes.push(format!(
                    "Dependency `{}` resolves to version `{}` with license `{}`.",
                    package.name, package.version, license
                ));
            } else {
                notes.push(format!(
                    "Dependency `{}` resolves to version `{}` but does not declare a license in Cargo metadata.",
                    package.name, package.version
                ));
            }

            if let Some(source) = &package.source
                && source.starts_with("git+")
            {
                notes.push(format!(
                    "Dependency `{}` resolves from a git source: `{}`.",
                    package.name, source
                ));
            }
        } else {
            notes.push(format!(
                "Dependency `{}` could not be resolved from `cargo metadata`; verify that the patched checkout is still resolvable.",
                change.name
            ));
        }
    }

    Ok(notes)
}

fn extract_dependency_changes(file: &crate::diff::ChangedFile) -> Vec<DependencyChange> {
    let mut current_section: Option<String> = None;
    let mut changes = Vec::new();

    for hunk in &file.hunks {
        for line in &hunk.lines {
            if matches!(line.kind, DiffLineKind::Context | DiffLineKind::Addition)
                && let Some(section) = parse_toml_section(&line.content)
            {
                current_section = Some(section.to_string());
                continue;
            }

            if line.kind == DiffLineKind::Addition
                && current_section
                    .as_deref()
                    .is_some_and(is_dependency_section)
                && let Some((dependency_name, dependency_spec)) =
                    parse_dependency_entry(&line.content)
            {
                changes.push(DependencyChange {
                    manifest_path: file.display_path(),
                    name: dependency_name.to_string(),
                    spec: dependency_spec.to_string(),
                });
            }
        }
    }

    changes
}

async fn load_cargo_metadata_packages(
    execution_repo_path: &std::path::Path,
) -> Result<Vec<CargoMetadataPackage>> {
    let output = Command::new("cargo")
        .args(["metadata", "--format-version", "1", "--locked"])
        .current_dir(execution_repo_path)
        .output()
        .await?;

    if !output.status.success() {
        anyhow::bail!(String::from_utf8_lossy(&output.stderr).trim().to_string());
    }

    let metadata: CargoMetadata = serde_json::from_slice(&output.stdout)?;
    Ok(metadata.packages)
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

fn parse_dependency_entry(line: &str) -> Option<(&str, &str)> {
    let trimmed = line.trim();
    if trimmed.starts_with('#') || !trimmed.contains('=') {
        return None;
    }

    let mut parts = trimmed.splitn(2, '=');
    let name = parts.next()?.trim();
    let spec = parts.next()?.trim();

    if name.is_empty() || spec.is_empty() {
        None
    } else {
        Some((name, spec))
    }
}

#[derive(Debug, Clone)]
struct DependencyChange {
    manifest_path: String,
    name: String,
    spec: String,
}

#[derive(Debug, Deserialize)]
struct CargoMetadata {
    packages: Vec<CargoMetadataPackage>,
}

#[derive(Debug, Deserialize)]
struct CargoMetadataPackage {
    name: String,
    version: String,
    license: Option<String>,
    source: Option<String>,
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
    use super::{detect_dependency_notes, parse_dependency_entry};
    use crate::diff::{ChangedFile, DiffHunk, DiffLine, DiffLineKind, FileChangeType, ParsedDiff};

    #[tokio::test]
    async fn reports_added_cargo_dependencies() {
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

        let notes =
            detect_dependency_notes(&parsed_diff, std::path::Path::new("/tmp/does-not-matter"))
                .await
                .expect("dependency scan should succeed");
        assert!(
            notes
                .iter()
                .any(|note| note == "New Rust dependency added in `Cargo.toml`: `serde`.")
        );
    }

    #[test]
    fn parses_dependency_name_and_spec() {
        assert_eq!(
            parse_dependency_entry("serde = { version = \"1\", features = [\"derive\"] }")
                .map(|(name, spec)| (name.to_string(), spec.to_string())),
            Some((
                "serde".to_string(),
                "{ version = \"1\", features = [\"derive\"] }".to_string()
            ))
        );
    }
}
