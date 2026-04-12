use anyhow::Result;
use clap::Parser;

/// VeriPatch CLI — verify AI-generated code changes from the command line.
#[derive(Parser, Debug)]
#[command(name = "veripatch", version, about)]
struct Cli {
    /// Path to the repository to verify against.
    #[arg(short, long)]
    repo: Option<String>,

    /// Path to a .patch file to verify.
    #[arg(short, long)]
    patch: Option<String>,

    /// Read diff from stdin.
    #[arg(long)]
    stdin: bool,
}

fn main() -> Result<()> {
    tracing_subscriber::fmt::init();

    let cli = Cli::parse();
    tracing::info!(?cli, "VeriPatch CLI started");

    // TODO: Implement CLI verification workflow
    println!("VeriPatch CLI — AI Output Verifier for Codebases");

    Ok(())
}
