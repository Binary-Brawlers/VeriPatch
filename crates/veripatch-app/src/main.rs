use anyhow::Result;

fn main() -> Result<()> {
    tracing_subscriber::fmt::init();
    tracing::info!("Starting VeriPatch");

    // TODO: Initialize GPUI application
    println!("VeriPatch — AI Output Verifier for Codebases");

    Ok(())
}
