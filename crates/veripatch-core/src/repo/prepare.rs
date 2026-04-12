use std::{
    path::{Path, PathBuf},
    process::Stdio,
};

use anyhow::Result;
use tempfile::TempDir;
use tokio::{io::AsyncWriteExt, process::Command};

use super::git::{ensure_git_repository, ensure_head_exists, load_local_diff};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VerificationMode {
    CurrentWorkingTree,
    ApplyPatchToTempClone,
}

pub(crate) struct PreparedRepository {
    execution_path: PathBuf,
    _temp_dir: Option<TempDir>,
}

impl PreparedRepository {
    pub(crate) fn execution_path(&self) -> &Path {
        &self.execution_path
    }
}

pub(crate) async fn prepare_repository(
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
