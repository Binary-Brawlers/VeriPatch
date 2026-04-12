//! Repository preparation helpers for verification.

use std::{
    path::{Path, PathBuf},
    process::Stdio,
};

use anyhow::Result;
use tempfile::TempDir;
use tokio::{io::AsyncWriteExt, process::Command};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VerificationMode {
    CurrentWorkingTree,
    ApplyPatchToTempClone,
}

pub struct PreparedRepository {
    execution_path: PathBuf,
    _temp_dir: Option<TempDir>,
}

impl PreparedRepository {
    pub fn execution_path(&self) -> &Path {
        &self.execution_path
    }
}

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

pub async fn prepare_repository(
    repo_path: &Path,
    diff_text: &str,
    mode: VerificationMode,
) -> Result<PreparedRepository> {
    match mode {
        VerificationMode::CurrentWorkingTree => Ok(PreparedRepository {
            execution_path: repo_path.to_path_buf(),
            _temp_dir: None,
        }),
        VerificationMode::ApplyPatchToTempClone => prepare_temp_clone(repo_path, diff_text).await,
    }
}

async fn prepare_temp_clone(repo_path: &Path, diff_text: &str) -> Result<PreparedRepository> {
    ensure_git_repository(repo_path).await?;
    ensure_head_exists(repo_path).await?;

    let current_worktree_diff = load_local_diff(repo_path).await.ok();

    let temp_dir = tempfile::tempdir()?;
    let clone_path = temp_dir.path().join("repo");
    let repo_path_text = repo_path.to_string_lossy().to_string();
    let clone_path_text = clone_path.to_string_lossy().to_string();

    run_command(
        repo_path,
        "git",
        &[
            "clone",
            "--shared",
            "--quiet",
            &repo_path_text,
            &clone_path_text,
        ],
    )
    .await?;

    run_command(
        &clone_path,
        "git",
        &["checkout", "--quiet", "--detach", "HEAD"],
    )
    .await?;

    if let Some(current_worktree_diff) = current_worktree_diff {
        apply_patch_to_clone(&clone_path, &current_worktree_diff).await?;
    }

    apply_patch_to_clone(&clone_path, diff_text).await?;

    Ok(PreparedRepository {
        execution_path: clone_path,
        _temp_dir: Some(temp_dir),
    })
}

async fn apply_patch_to_clone(repo_path: &Path, diff_text: &str) -> Result<()> {
    let mut child = Command::new("git")
        .args(["apply", "--whitespace=nowarn", "-"])
        .current_dir(repo_path)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;

    if let Some(mut stdin) = child.stdin.take() {
        stdin.write_all(diff_text.as_bytes()).await?;
    }

    let output = child.wait_with_output().await?;
    if output.status.success() {
        Ok(())
    } else {
        anyhow::bail!(String::from_utf8_lossy(&output.stderr).trim().to_string())
    }
}

async fn ensure_git_repository(repo_path: &Path) -> Result<()> {
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

async fn ensure_head_exists(repo_path: &Path) -> Result<()> {
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

async fn run_command(repo_path: &Path, program: &str, args: &[&str]) -> Result<()> {
    let output = Command::new(program)
        .args(args)
        .current_dir(repo_path)
        .output()
        .await?;

    if output.status.success() {
        Ok(())
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
