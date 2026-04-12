use std::path::Path;

use anyhow::Result;
use tokio::process::Command;

pub async fn load_local_diff(repo_path: &Path) -> Result<String> {
    ensure_git_repository(repo_path).await?;

    let mut diff_parts = Vec::new();

    if git_head_exists(repo_path).await? {
        let tracked_diff = run_git_capture(repo_path, &["diff", "HEAD", "--binary"]).await?;
        if !tracked_diff.trim().is_empty() {
            diff_parts.push(tracked_diff);
        }
    } else {
        let staged_diff = run_git_capture(repo_path, &["diff", "--cached", "--binary"]).await?;
        if !staged_diff.trim().is_empty() {
            diff_parts.push(staged_diff);
        }

        let unstaged_diff = run_git_capture(repo_path, &["diff", "--binary"]).await?;
        if !unstaged_diff.trim().is_empty() {
            diff_parts.push(unstaged_diff);
        }
    }

    for untracked_file in list_untracked_files(repo_path).await? {
        let file_diff = build_untracked_file_diff(repo_path, &untracked_file).await?;
        if !file_diff.trim().is_empty() {
            diff_parts.push(file_diff);
        }
    }

    let combined = diff_parts.join("\n");
    if combined.trim().is_empty() {
        anyhow::bail!("no local staged, unstaged, or untracked changes were found to verify");
    }

    Ok(combined)
}

pub(super) async fn ensure_git_repository(repo_path: &Path) -> Result<()> {
    let output = Command::new("git")
        .args(["rev-parse", "--show-toplevel"])
        .current_dir(repo_path)
        .output()
        .await?;

    if output.status.success() {
        Ok(())
    } else {
        anyhow::bail!(String::from_utf8_lossy(&output.stderr).trim().to_string())
    }
}

pub(super) async fn ensure_head_exists(repo_path: &Path) -> Result<()> {
    if git_head_exists(repo_path).await? {
        Ok(())
    } else {
        anyhow::bail!(
            "cannot prepare a temporary verification checkout because this repository has no HEAD commit"
        )
    }
}

async fn git_head_exists(repo_path: &Path) -> Result<bool> {
    let output = Command::new("git")
        .args(["rev-parse", "--verify", "HEAD"])
        .current_dir(repo_path)
        .output()
        .await?;

    Ok(output.status.success())
}

async fn run_git_capture(repo_path: &Path, args: &[&str]) -> Result<String> {
    let output = Command::new("git")
        .args(args)
        .current_dir(repo_path)
        .output()
        .await?;

    if output.status.success() {
        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    } else {
        anyhow::bail!(String::from_utf8_lossy(&output.stderr).trim().to_string())
    }
}

async fn list_untracked_files(repo_path: &Path) -> Result<Vec<String>> {
    let output = Command::new("git")
        .args(["ls-files", "--others", "--exclude-standard", "-z"])
        .current_dir(repo_path)
        .output()
        .await?;

    if !output.status.success() {
        anyhow::bail!(String::from_utf8_lossy(&output.stderr).trim().to_string());
    }

    Ok(output
        .stdout
        .split(|byte| *byte == 0)
        .filter(|entry| !entry.is_empty())
        .map(|entry| String::from_utf8_lossy(entry).to_string())
        .collect())
}

async fn build_untracked_file_diff(repo_path: &Path, relative_path: &str) -> Result<String> {
    let output = Command::new("git")
        .args([
            "diff",
            "--no-index",
            "--binary",
            "--",
            "/dev/null",
            relative_path,
        ])
        .current_dir(repo_path)
        .output()
        .await?;

    if output.status.success() || output.status.code() == Some(1) {
        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    } else {
        anyhow::bail!(String::from_utf8_lossy(&output.stderr).trim().to_string())
    }
}
