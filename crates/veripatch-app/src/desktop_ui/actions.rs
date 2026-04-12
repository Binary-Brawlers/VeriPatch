use anyhow::{Context as _, Result};
use gpui::{AppContext, Context};

use super::{execute_request, types::*};

impl DesktopState {
    pub(super) fn select_repo(&mut self, cx: &mut Context<Self>) {
        let mut dialog = rfd::FileDialog::new();
        if self.repo_path.exists() {
            dialog = dialog.set_directory(&self.repo_path);
        }

        if let Some(path) = dialog.pick_folder() {
            self.repo_path = path;
            self.run_state = RunState::Idle;
            cx.notify();
        }
    }

    pub(super) fn select_input_source(&mut self, source: InputSource, cx: &mut Context<Self>) {
        self.input_source = source;
        cx.notify();
    }

    pub(super) fn capture_clipboard_diff(&mut self, cx: &mut Context<Self>) {
        match cx.read_from_clipboard().and_then(|item| item.text()) {
            Some(text) if !text.trim().is_empty() => {
                self.clipboard_diff = Some(text);
                self.input_source = InputSource::ClipboardDiff;
                self.run_state = RunState::Idle;
            }
            _ => {
                self.run_state = RunState::Failed(
                    "Clipboard is empty or does not contain a unified diff yet.".to_string(),
                );
            }
        }

        cx.notify();
    }

    pub(super) fn select_patch_file(&mut self, cx: &mut Context<Self>) {
        let mut dialog = rfd::FileDialog::new().add_filter("Patch", &["patch", "diff"]);
        if self.repo_path.exists() {
            dialog = dialog.set_directory(&self.repo_path);
        }

        if let Some(path) = dialog.pick_file() {
            self.patch_path = Some(path);
            self.input_source = InputSource::PatchFile;
            self.run_state = RunState::Idle;
            cx.notify();
        }
    }

    pub(super) fn start_verification(&mut self, cx: &mut Context<Self>) {
        if matches!(self.run_state, RunState::Running) {
            return;
        }

        let request = match self.build_request() {
            Ok(request) => request,
            Err(error) => {
                self.run_state = RunState::Failed(format!("{error:#}"));
                cx.notify();
                return;
            }
        };

        self.run_state = RunState::Running;
        cx.notify();

        cx.spawn(async move |this, cx| {
            let outcome = cx
                .background_spawn(async move { execute_request(request) })
                .await;

            this.update(cx, |state, cx| {
                state.run_state = match outcome {
                    Ok(snapshot) => RunState::Finished(snapshot),
                    Err(error) => RunState::Failed(format!("{error:#}")),
                };
                cx.notify();
            })
            .ok();
        })
        .detach();
    }

    fn build_request(&self) -> Result<VerificationRequest> {
        let source = match self.input_source {
            InputSource::CurrentWorkingTree => VerificationRequestSource::CurrentWorkingTree,
            InputSource::ClipboardDiff => {
                VerificationRequestSource::ClipboardDiff(self.clipboard_diff.clone().context(
                    "Load a unified diff from the clipboard before running verification",
                )?)
            }
            InputSource::PatchFile => VerificationRequestSource::PatchFile(
                self.patch_path
                    .clone()
                    .context("Choose a .patch or .diff file before running verification")?,
            ),
        };

        Ok(VerificationRequest {
            repo_path: self.repo_path.clone(),
            source,
        })
    }
}
