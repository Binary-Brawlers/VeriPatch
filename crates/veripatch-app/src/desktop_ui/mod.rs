mod actions;
mod components;
mod types;
mod view;

use anyhow::{Context as _, Result};
use clap::Parser;
use gpui::{AppContext, Application, Bounds, WindowBounds, WindowOptions, px, size};
use std::{fs, path::PathBuf};
use veripatch_core::{VerificationInput, VerificationMode, load_local_diff, verify};

use types::{
    Cli, DesktopState, VerificationRequest, VerificationRequestSource, VerificationSnapshot,
};

pub fn run() -> Result<()> {
    tracing_subscriber::fmt::init();

    let cli = Cli::parse();
    let repo_path = resolve_repo_path(cli.repo)?;

    Application::with_platform(gpui_platform::current_platform(false)).run(move |cx| {
        let bounds = Bounds::centered(None, size(px(1120.0), px(860.0)), cx);
        cx.open_window(
            WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(bounds)),
                ..Default::default()
            },
            move |_, cx| cx.new(|_| DesktopState::new(repo_path.clone())),
        )
        .expect("failed to open VeriPatch window");
    });

    Ok(())
}

fn resolve_repo_path(repo: Option<String>) -> Result<PathBuf> {
    match repo {
        Some(path) => Ok(PathBuf::from(path)),
        None => Ok(std::env::current_dir()?),
    }
}

fn execute_request(request: VerificationRequest) -> Result<VerificationSnapshot> {
    let runtime = tokio::runtime::Runtime::new()?;

    runtime.block_on(async move {
        let repo_path = request.repo_path.clone();
        let (diff_text, mode, source_label) = match request.source {
            VerificationRequestSource::CurrentWorkingTree => (
                load_local_diff(&request.repo_path).await?,
                VerificationMode::CurrentWorkingTree,
                "Current working tree".to_string(),
            ),
            VerificationRequestSource::ClipboardDiff(diff_text) => (
                diff_text,
                VerificationMode::ApplyPatchToTempClone,
                "Clipboard diff".to_string(),
            ),
            VerificationRequestSource::PatchFile(path) => (
                fs::read_to_string(&path)
                    .with_context(|| format!("failed to read patch file `{}`", path.display()))?,
                VerificationMode::ApplyPatchToTempClone,
                format!("Patch file: {}", path.display()),
            ),
        };

        let result = verify(VerificationInput {
            repo_path,
            diff_text,
            mode,
        })
        .await?;

        Ok(VerificationSnapshot {
            source_label,
            result,
        })
    })
}
