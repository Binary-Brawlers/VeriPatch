//! Runner trait and common types.

mod command;
mod rust;

use std::path::PathBuf;

use anyhow::Result;
use serde::{Deserialize, Serialize};

use command::skipped_check;

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
        return rust::run_rust_checks(context, changed_paths).await;
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
