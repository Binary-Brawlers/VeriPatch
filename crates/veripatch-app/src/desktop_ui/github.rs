use std::path::Path;

use anyhow::{Context, Result};
use serde::Deserialize;
use tokio::process::Command;

use super::types::PullRequestSummary;

pub async fn list_pull_requests(repo_path: &Path) -> Result<Vec<PullRequestSummary>> {
    let stdout = run_gh_capture(
        repo_path,
        &[
            "pr",
            "list",
            "--state",
            "open",
            "--limit",
            "50",
            "--json",
            "number,title,baseRefName,headRefName,updatedAt,isDraft,author",
        ],
    )
    .await?;

    let entries: Vec<GhPullRequest> =
        serde_json::from_str(&stdout).context("failed to parse `gh pr list` response")?;

    Ok(entries
        .into_iter()
        .map(|entry| PullRequestSummary {
            number: entry.number,
            title: entry.title,
            author: entry
                .author
                .and_then(|author| author.login)
                .unwrap_or_else(|| "unknown".to_string()),
            base_ref_name: entry.base_ref_name,
            head_ref_name: entry.head_ref_name,
            updated_at: entry.updated_at,
            is_draft: entry.is_draft,
        })
        .collect())
}

pub async fn load_pull_request_diff(repo_path: &Path, number: u64) -> Result<String> {
    run_gh_capture(repo_path, &["pr", "diff", &number.to_string()]).await
}

pub async fn merge_pull_request(repo_path: &Path, number: u64) -> Result<()> {
    run_gh(repo_path, &["pr", "merge", &number.to_string(), "--merge"]).await
}

pub async fn close_pull_request(repo_path: &Path, number: u64) -> Result<()> {
    run_gh(repo_path, &["pr", "close", &number.to_string()]).await
}

async fn run_gh(repo_path: &Path, args: &[&str]) -> Result<()> {
    let output = Command::new("gh")
        .args(args)
        .current_dir(repo_path)
        .output()
        .await
        .map_err(map_gh_spawn_error)?;

    if output.status.success() {
        Ok(())
    } else {
        anyhow::bail!(format_gh_failure(&output.stderr))
    }
}

async fn run_gh_capture(repo_path: &Path, args: &[&str]) -> Result<String> {
    let output = Command::new("gh")
        .args(args)
        .current_dir(repo_path)
        .output()
        .await
        .map_err(map_gh_spawn_error)?;

    if output.status.success() {
        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    } else {
        anyhow::bail!(format_gh_failure(&output.stderr))
    }
}

fn map_gh_spawn_error(error: std::io::Error) -> anyhow::Error {
    anyhow::anyhow!(
        "GitHub CLI (`gh`) is required for pull request workflows. Install it and run `gh auth login`. Underlying error: {error}"
    )
}

fn format_gh_failure(stderr: &[u8]) -> String {
    let message = String::from_utf8_lossy(stderr).trim().to_string();
    if message.is_empty() {
        "GitHub CLI command failed without an error message".to_string()
    } else {
        message
    }
}

#[derive(Debug, Deserialize)]
struct GhPullRequest {
    number: u64,
    title: String,
    #[serde(rename = "baseRefName")]
    base_ref_name: String,
    #[serde(rename = "headRefName")]
    head_ref_name: String,
    #[serde(rename = "updatedAt")]
    updated_at: String,
    #[serde(rename = "isDraft")]
    is_draft: bool,
    author: Option<GhActor>,
}

#[derive(Debug, Deserialize)]
struct GhActor {
    login: Option<String>,
}
