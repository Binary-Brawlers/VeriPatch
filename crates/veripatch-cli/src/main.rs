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
    let output = Command::new("git")
        .arg("diff")
        .arg("HEAD")
        .current_dir(repo_path)
        .output()
        .await?;

    if !output.status.success() {
        anyhow::bail!(String::from_utf8_lossy(&output.stderr).trim().to_string());
    }

    Ok(String::from_utf8_lossy(&output.stdout).to_string())
}
