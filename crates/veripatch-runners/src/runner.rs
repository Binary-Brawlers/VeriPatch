//! Runner trait and common types.

use std::{
    collections::BTreeSet,
    fs,
    path::{Path, PathBuf},
};

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

pub async fn run_default_checks(
    context: &RunnerContext,
    changed_paths: &[String],
) -> Result<Vec<CheckResult>> {
    if context.repo_path.join("Cargo.toml").exists() {
        let mut results = Vec::with_capacity(3);
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

        results.push(run_rust_tests(context, changed_paths).await?);

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
        skipped_check(
            "Impacted Tests",
            "no supported project manifest was detected for the MVP runners",
        ),
    ])
}

async fn run_rust_tests(context: &RunnerContext, changed_paths: &[String]) -> Result<CheckResult> {
    match build_rust_test_plan(&context.repo_path, changed_paths)? {
        RustTestPlan::Full { reason } => {
            let mut result =
                run_command_check(context, "Impacted Tests", "cargo", &["test", "--quiet"]).await?;
            if !reason.is_empty() {
                result.summary = format!("{} ({reason})", result.summary);
            }
            Ok(result)
        }
        RustTestPlan::Targets(targets) => run_test_targets(context, &targets).await,
    }
}

fn build_rust_test_plan(repo_path: &Path, changed_paths: &[String]) -> Result<RustTestPlan> {
    if changed_paths
        .iter()
        .any(|path| path == "Cargo.toml" || path.ends_with("/Cargo.toml") || path == "Cargo.lock")
    {
        return Ok(RustTestPlan::Full {
            reason: "manifest changes require the broader Rust test suite".to_string(),
        });
    }

    let available_targets = discover_rust_test_targets(repo_path.join("tests"))?;
    if available_targets.is_empty() {
        return Ok(RustTestPlan::Full {
            reason: "no integration test targets were discovered, so the full test suite was used"
                .to_string(),
        });
    }

    let targets = select_impacted_test_targets(changed_paths, &available_targets);
    if targets.is_empty() {
        Ok(RustTestPlan::Full {
            reason: "could not confidently map changed files to specific integration tests"
                .to_string(),
        })
    } else if targets.len() > 4 {
        Ok(RustTestPlan::Full {
            reason: format!(
                "{} related test targets were detected, so the full suite was used instead",
                targets.len()
            ),
        })
    } else {
        Ok(RustTestPlan::Targets(targets))
    }
}

fn discover_rust_test_targets(tests_dir: PathBuf) -> Result<Vec<String>> {
    if !tests_dir.exists() {
        return Ok(Vec::new());
    }

    let mut targets = BTreeSet::new();
    collect_rust_test_targets(&tests_dir, &tests_dir, &mut targets)?;
    Ok(targets.into_iter().collect())
}

fn collect_rust_test_targets(
    root: &Path,
    current: &Path,
    targets: &mut BTreeSet<String>,
) -> Result<()> {
    for entry in fs::read_dir(current)? {
        let entry = entry?;
        let path = entry.path();
        let file_type = entry.file_type()?;

        if file_type.is_dir() {
            let mod_rs = path.join("mod.rs");
            if mod_rs.exists() {
                if let Ok(relative) = path.strip_prefix(root) {
                    let target = relative.to_string_lossy().replace('/', "::");
                    if !target.is_empty() {
                        targets.insert(target);
                    }
                }
            }

            collect_rust_test_targets(root, &path, targets)?;
            continue;
        }

        if path.extension().and_then(|ext| ext.to_str()) == Some("rs") {
            if path.file_name().and_then(|name| name.to_str()) == Some("mod.rs") {
                continue;
            }

            if let Some(stem) = path.file_stem().and_then(|stem| stem.to_str()) {
                targets.insert(stem.to_string());
            }
        }
    }

    Ok(())
}

fn select_impacted_test_targets(
    changed_paths: &[String],
    available_targets: &[String],
) -> Vec<String> {
    let mut matched = BTreeSet::new();

    for changed_path in changed_paths {
        let normalized = changed_path.replace('\\', "/");

        if let Some(target) = changed_path_to_test_target(&normalized) {
            if available_targets
                .iter()
                .any(|candidate| candidate == &target)
            {
                matched.insert(target);
            }
        }

        if let Some(stem) = Path::new(&normalized)
            .file_stem()
            .and_then(|stem| stem.to_str())
        {
            for candidate in available_targets {
                if candidate == stem || candidate.contains(stem) || stem.contains(candidate) {
                    matched.insert(candidate.clone());
                }
            }
        }
    }

    matched.into_iter().collect()
}

fn changed_path_to_test_target(changed_path: &str) -> Option<String> {
    let stripped = changed_path.strip_prefix("tests/")?;
    if let Some(parent) = stripped.strip_suffix("/mod.rs") {
        return Some(parent.replace('/', "::"));
    }

    if let Some(file) = stripped.strip_suffix(".rs") {
        return Some(file.rsplit('/').next()?.to_string());
    }

    None
}

async fn run_test_targets(context: &RunnerContext, targets: &[String]) -> Result<CheckResult> {
    let mut outputs = Vec::new();
    let mut commands = Vec::new();
    let mut failed = false;

    for target in targets {
        let args = vec![
            "test".to_string(),
            "--quiet".to_string(),
            "--test".to_string(),
            target.clone(),
        ];
        let command = format!("cargo {}", args.join(" "));
        commands.push(command.clone());

        let output = Command::new("cargo")
            .args(&args)
            .current_dir(&context.repo_path)
            .output()
            .await?;

        let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
        let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
        let details = combine_details(&stdout, &stderr)
            .unwrap_or_else(|| "Command completed successfully.".to_string());

        if !output.status.success() {
            failed = true;
        }

        outputs.push(format!("{command}\n{details}"));
    }

    Ok(CheckResult {
        name: "Impacted Tests".to_string(),
        status: if failed {
            CheckStatus::Failed
        } else {
            CheckStatus::Passed
        },
        command: Some(commands.join("; ")),
        summary: format!(
            "Ran {} impacted integration test target(s): {}.",
            targets.len(),
            targets.join(", ")
        ),
        details: Some(outputs.join("\n\n")),
    })
}

enum RustTestPlan {
    Full { reason: String },
    Targets(Vec<String>),
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

#[cfg(test)]
mod tests {
    use super::select_impacted_test_targets;

    #[test]
    fn matches_related_rust_test_targets() {
        let changed_paths = vec!["src/report.rs".to_string(), "tests/report.rs".to_string()];
        let available_targets = vec![
            "report".to_string(),
            "pipeline".to_string(),
            "report_markdown".to_string(),
        ];

        let selected = select_impacted_test_targets(&changed_paths, &available_targets);
        assert_eq!(
            selected,
            vec!["report".to_string(), "report_markdown".to_string()]
        );
    }
}
