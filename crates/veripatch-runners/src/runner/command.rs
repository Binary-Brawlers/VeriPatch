use std::ffi::OsStr;

use anyhow::Result;
use tokio::process::Command;

use super::{CheckResult, CheckStatus, RunnerContext};

pub(super) async fn run_command_check(
    context: &RunnerContext,
    name: &str,
    program: impl AsRef<OsStr>,
    args: &[&str],
) -> Result<CheckResult> {
    let args = args.iter().map(|arg| arg.to_string()).collect::<Vec<_>>();
    run_command_check_owned(context, name, program, &args).await
}

pub(super) async fn run_command_check_owned(
    context: &RunnerContext,
    name: &str,
    program: impl AsRef<OsStr>,
    args: &[String],
) -> Result<CheckResult> {
    let program = program.as_ref();
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
        command: Some(format!("{} {}", program.to_string_lossy(), args.join(" "))),
        summary: summarize_output(output.status.success(), &stdout, &stderr),
        details,
    })
}

pub(super) fn skipped_check(name: &str, reason: &str) -> CheckResult {
    CheckResult {
        name: name.to_string(),
        status: CheckStatus::Skipped,
        command: None,
        summary: reason.to_string(),
        details: None,
    }
}

pub(super) fn combine_details(stdout: &str, stderr: &str) -> Option<String> {
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

fn first_non_empty_line(text: &str) -> Option<String> {
    text.lines()
        .map(str::trim)
        .find(|line| !line.is_empty())
        .map(ToOwned::to_owned)
}
