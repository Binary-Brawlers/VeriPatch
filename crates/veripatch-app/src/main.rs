#[cfg(feature = "desktop-ui")]
mod desktop_ui;

#[cfg(feature = "desktop-ui")]
fn main() -> anyhow::Result<()> {
    desktop_ui::run()
}

#[cfg(not(feature = "desktop-ui"))]
fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();
    println!(
        "VeriPatch desktop UI is disabled in this build. Enable the `desktop-ui` feature in an environment with the required GPUI platform toolchain."
    );
    Ok(())
}
