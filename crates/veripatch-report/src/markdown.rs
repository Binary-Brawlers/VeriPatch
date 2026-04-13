//! Markdown report renderer.

use anyhow::Result;
use veripatch_core::{Verdict, VerificationResult};
use veripatch_rules::rule::RiskSeverity;
use veripatch_runners::runner::CheckStatus;

pub fn render_markdown(result: &VerificationResult) -> Result<String> {
    let mut output = String::new();

    output.push_str("# VeriPatch Report\n\n");
    output.push_str(&format!("- Repository: `{}`\n", result.repo_path.display()));
    output.push_str(&format!(
        "- Verdict: **{}**\n",
        verdict_label(result.verdict)
    ));
    output.push_str(&format!("- Score: **{} / 100**\n", result.score));
    output.push_str(&format!(
        "- Diff Scope: **{} file(s)**, **{} additions**, **{} deletions**\n\n",
        result.diff.files.len(),
        result.diff.total_additions,
        result.diff.total_deletions
    ));

    output.push_str("## Changed Files\n\n");
    for file in &result.diff.files {
        output.push_str(&format!(
            "- `{}` ({:?}, +{}, -{})\n",
            file.display_path(),
            file.change_type,
            file.additions,
            file.deletions
        ));
    }

    output.push_str("\n## Checks\n\n");
    for check in &result.checks {
        output.push_str(&format!(
            "- [{}] **{}**: {}\n",
            check_status_label(check.status),
            check.name,
            check.summary
        ));

        if let Some(command) = &check.command {
            output.push_str(&format!("  Command: `{command}`\n"));
        }
    }

    if !result.risky_patterns.is_empty() {
        output.push_str("\n## Risky Patterns\n\n");
        for finding in &result.risky_patterns {
            output.push_str(&format!(
                "- [{}] {}{}\n",
                severity_label(finding.severity),
                finding.message,
                format_location(finding.file_path.as_deref(), finding.line_number)
            ));
        }
    }

    if !result.assumptions.is_empty() {
        output.push_str("\n## Assumptions\n\n");
        for assumption in &result.assumptions {
            output.push_str(&format!(
                "- {}{}\n",
                assumption.message,
                format_location(assumption.file_path.as_deref(), assumption.line_number)
            ));
        }
    }

    if !result.dependency_notes.is_empty() {
        output.push_str("\n## Dependency Review\n\n");
        for note in &result.dependency_notes {
            output.push_str(&format!("- {}\n", note));
        }
    }

    if !result.warnings.is_empty() {
        output.push_str("\n## Warnings\n\n");
        for warning in &result.warnings {
            output.push_str(&format!("- {}\n", warning));
        }
    }

    Ok(output)
}

fn verdict_label(verdict: Verdict) -> &'static str {
    match verdict {
        Verdict::Safe => "Safe",
        Verdict::Risky => "Risky",
        Verdict::Broken => "Broken",
    }
}

fn check_status_label(status: CheckStatus) -> &'static str {
    match status {
        CheckStatus::Passed => "pass",
        CheckStatus::Failed => "fail",
        CheckStatus::Skipped => "skip",
    }
}

fn severity_label(severity: RiskSeverity) -> &'static str {
    match severity {
        RiskSeverity::Low => "low",
        RiskSeverity::Medium => "medium",
        RiskSeverity::High => "high",
    }
}

fn format_location(file_path: Option<&str>, line_number: Option<usize>) -> String {
    match (file_path, line_number) {
        (Some(file_path), Some(line_number)) => format!(" (`{}:{}`)", file_path, line_number),
        (Some(file_path), None) => format!(" (`{}`)", file_path),
        _ => String::new(),
    }
}
