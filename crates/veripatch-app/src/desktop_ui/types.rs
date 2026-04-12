use clap::Parser;
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(name = "veripatch", version, about)]
pub(super) struct Cli {
    /// Path to the repository to verify against.
    #[arg(short, long)]
    pub(super) repo: Option<String>,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum InputSource {
    CurrentWorkingTree,
    ClipboardDiff,
    PatchFile,
}

pub(super) struct DesktopState {
    pub(super) repo_path: PathBuf,
    pub(super) input_source: InputSource,
    pub(super) clipboard_diff: Option<String>,
    pub(super) patch_path: Option<PathBuf>,
    pub(super) run_state: RunState,
}

pub(super) enum RunState {
    Idle,
    Running,
    Finished(VerificationSnapshot),
    Failed(String),
}

pub(super) struct VerificationSnapshot {
    pub(super) source_label: String,
    pub(super) result: veripatch_core::VerificationResult,
}

#[derive(Clone)]
pub(super) struct VerificationRequest {
    pub(super) repo_path: PathBuf,
    pub(super) source: VerificationRequestSource,
}

#[derive(Clone)]
pub(super) enum VerificationRequestSource {
    CurrentWorkingTree,
    ClipboardDiff(String),
    PatchFile(PathBuf),
}

impl DesktopState {
    pub(super) fn new(repo_path: PathBuf) -> Self {
        Self {
            repo_path,
            input_source: InputSource::CurrentWorkingTree,
            clipboard_diff: None,
            patch_path: None,
            run_state: RunState::Idle,
        }
    }
}
