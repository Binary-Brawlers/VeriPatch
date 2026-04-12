//! Verification pipeline orchestration.

mod dependencies;
mod scoring;

use std::path::PathBuf;

use anyhow::Result;
use veripatch_rules::rule::{RuleInputLine, analyze_lines, detect_assumptions};
use veripatch_runners::runner::{RunnerContext, run_default_checks};

use crate::{
    diff::{DiffLineKind, ParsedDiff, parse_unified_diff},
    repo::{VerificationMode, prepare_repository},
    verdict::VerificationResult,
};
use dependencies::detect_dependency_notes;
use scoring::{build_warnings, calculate_score, determine_verdict};

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
