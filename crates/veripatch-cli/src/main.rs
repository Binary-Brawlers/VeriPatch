use anyhow::Result;
use clap::Parser;
use std::{
    fs,
    io::{self, Read},
    path::PathBuf,
};
use tokio::process::Command;

use veripatch_core::{VerificationInput, verify};
use veripatch_report::render_markdown;

/// VeriPatch CLI — verify AI-generated code changes from the command line.
#[derive(Parser, Debug)]
#[command(name = "veripatch", version, about)]
struct Cli {
    /// Path to the repository to verify against.
    #[arg(short, long)]
    repo: Option<String>,

    /// Path to a .patch file to verify.
    #[arg(short, long, conflicts_with = "stdin")]
    patch: Option<String>,

    /// Read diff from stdin.
    #[arg(long, conflicts_with = "patch")]
    stdin: bool,
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();

    let cli = Cli::parse();

    let repo_path = resolve_repo_path(cli.repo.clone())?;
    let diff_text = load_diff_input(&cli, &repo_path).await?;

    let result = verify(VerificationInput {
        repo_path,
        diff_text,
    })
    .await?;
    let markdown = render_markdown(&result)?;

    println!("{markdown}");

    Ok(())
}

fn resolve_repo_path(repo: Option<String>) -> Result<PathBuf> {
    match repo {
        Some(path) => Ok(PathBuf::from(path)),
        None => Ok(std::env::current_dir()?),
    }
}

async fn load_diff_input(cli: &Cli, repo_path: &PathBuf) -> Result<String> {
    if let Some(path) = &cli.patch {
        return Ok(fs::read_to_string(path)?);
    }

    if cli.stdin {
        let mut buffer = String::new();
        io::stdin().read_to_string(&mut buffer)?;
        return Ok(buffer);
    }

    read_git_working_tree_diff(repo_path).await
}

async fn read_git_working_tree_diff(repo_path: &PathBuf) -> Result<String> {
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

async fn ensure_git_repository(repo_path: &PathBuf) -> Result<()> {
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

async fn git_head_exists(repo_path: &PathBuf) -> Result<bool> {
    let output = Command::new("git")
        .args(["rev-parse", "--verify", "HEAD"])
        .current_dir(repo_path)
        .output()
        .await?;

    Ok(output.status.success())
}

async fn run_git_capture(repo_path: &PathBuf, args: &[&str]) -> Result<String> {
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

async fn list_untracked_files(repo_path: &PathBuf) -> Result<Vec<String>> {
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

async fn build_untracked_file_diff(repo_path: &PathBuf, relative_path: &str) -> Result<String> {
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
