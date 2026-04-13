use anyhow::Result;
use clap::Parser;
use std::{
    fs,
    io::{self, Read},
    path::Path,
    path::PathBuf,
};

use veripatch_core::{VerificationInput, VerificationMode, load_local_diff, verify};
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
    let (diff_text, mode) = load_diff_input(&cli, &repo_path).await?;

    let result = verify(VerificationInput {
        repo_path,
        diff_text,
        mode,
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

async fn load_diff_input(cli: &Cli, repo_path: &Path) -> Result<(String, VerificationMode)> {
    if let Some(path) = &cli.patch {
        return Ok((
            fs::read_to_string(path)?,
            VerificationMode::ApplyPatchToTempClone,
        ));
    }

    if cli.stdin {
        let mut buffer = String::new();
        io::stdin().read_to_string(&mut buffer)?;
        return Ok((buffer, VerificationMode::ApplyPatchToTempClone));
    }

    Ok((
        load_local_diff(repo_path).await?,
        VerificationMode::CurrentWorkingTree,
    ))
}
