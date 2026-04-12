//! Runner trait and common types.

use std::path::{Path, PathBuf};

use anyhow::Result;
use serde::{Deserialize, Serialize};
use tokio::process::Command;

#[derive(Debug, Clone)]
pub struct RunnerContext {
    pub repo_path: PathBuf,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CheckStatus {
    Passed,
    Failed,
    Skipped,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CheckResult {
    pub name: String,
    pub status: CheckStatus,
    pub command: Option<String>,
    pub summary: String,
    pub details: Option<String>,
}

pub async fn run_default_checks(context: &RunnerContext) -> Result<Vec<CheckResult>> {
    if context.repo_path.join("Cargo.toml").exists() {
        let mut results = Vec::with_capacity(2);
        results.push(
            run_command_check(
                context,
                "Compile / Type Check",
                "cargo",
                &["check", "--quiet"],
            )
            .await?,
        );

        if cargo_supports_clippy(&context.repo_path).await? {
            results.push(
                run_command_check(
                    context,
                    "Lint",
                    "cargo",
                    &["clippy", "--quiet", "--all-targets", "--no-deps"],
                )
                .await?,
            );
        } else {
            results.push(skipped_check(
                "Lint",
                "cargo-clippy is not installed in this environment",
            ));
        }

        return Ok(results);
    }

    Ok(vec![
        skipped_check(
            "Compile / Type Check",
            "no supported project manifest was detected for the MVP runners",
        ),
        skipped_check(
            "Lint",
            "no supported project manifest was detected for the MVP runners",
        ),
    ])
}

async fn cargo_supports_clippy(repo_path: &Path) -> Result<bool> {
    let output = Command::new("cargo")
        .arg("clippy")
        .arg("--version")
        .current_dir(repo_path)
        .output()
        .await?;

    Ok(output.status.success())
}

async fn run_command_check(
    context: &RunnerContext,
    name: &str,
    program: &str,
    args: &[&str],
) -> Result<CheckResult> {
    let output = Command::new(program)
        .args(args)
        .current_dir(&context.repo_path)
        .output()
        .await?;

    let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
    let details = combine_details(&stdout, &stderr);

    Ok(CheckResult {
        name: name.to_string(),
        status: if output.status.success() {
            CheckStatus::Passed
        } else {
            CheckStatus::Failed
        },
        command: Some(format!("{} {}", program, args.join(" "))),
        summary: summarize_output(output.status.success(), &stdout, &stderr),
        details,
    })
}

fn skipped_check(name: &str, reason: &str) -> CheckResult {
    CheckResult {
        name: name.to_string(),
        status: CheckStatus::Skipped,
        command: None,
        summary: reason.to_string(),
        details: None,
    }
}

fn summarize_output(success: bool, stdout: &str, stderr: &str) -> String {
    if success {
        if stderr.is_empty() && stdout.is_empty() {
            "Command completed successfully.".to_string()
        } else {
            first_non_empty_line(stderr)
                .or_else(|| first_non_empty_line(stdout))
                .unwrap_or_else(|| "Command completed successfully.".to_string())
        }
    } else {
        first_non_empty_line(stderr)
            .or_else(|| first_non_empty_line(stdout))
            .unwrap_or_else(|| "Command failed without additional output.".to_string())
    }
}

fn combine_details(stdout: &str, stderr: &str) -> Option<String> {
    let trimmed_stdout = stdout.trim();
    let trimmed_stderr = stderr.trim();

    match (trimmed_stdout.is_empty(), trimmed_stderr.is_empty()) {
        (true, true) => None,
        (false, true) => Some(trimmed_stdout.to_string()),
        (true, false) => Some(trimmed_stderr.to_string()),
        (false, false) => Some(format!(
            "stdout:\n{}\n\nstderr:\n{}",
            trimmed_stdout, trimmed_stderr
        )),
    }
}

fn first_non_empty_line(text: &str) -> Option<String> {
    text.lines()
        .map(str::trim)
        .find(|line| !line.is_empty())
        .map(ToOwned::to_owned)
}
