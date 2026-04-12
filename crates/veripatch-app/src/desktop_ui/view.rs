use gpui::{Context, FontWeight, Render, Window, div, prelude::*, rgb};

use super::{
    components::{button, panel, render_snapshot},
    types::{DesktopState, InputSource, RunState},
};

impl Render for DesktopState {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let clipboard_summary = self
            .clipboard_diff
            .as_ref()
            .map(|diff| format!("{} line(s) loaded from the clipboard", diff.lines().count()))
            .unwrap_or_else(|| "No clipboard diff loaded yet".to_string());

        let patch_summary = self
            .patch_path
            .as_ref()
            .map(|path| path.display().to_string())
            .unwrap_or_else(|| "No patch file selected yet".to_string());

        let result_panel = match &self.run_state {
            RunState::Idle => panel()
                .flex()
                .flex_col()
                .gap_3()
                .child(
                    div()
                        .text_lg()
                        .font_weight(FontWeight::SEMIBOLD)
                        .child("Ready to verify"),
                )
                .child(
                    div()
                        .text_sm()
                        .text_color(rgb(0x94a3b8))
                        .child(
                            "Pick a repository, choose a diff source, and run the verification pipeline.",
                        ),
                ),
            RunState::Running => panel()
                .flex()
                .flex_col()
                .gap_3()
                .child(
                    div()
                        .text_lg()
                        .font_weight(FontWeight::SEMIBOLD)
                        .child("Running verification"),
                )
                .child(
                    div()
                        .text_sm()
                        .text_color(rgb(0x94a3b8))
                        .child(
                            "Compile checks, lint, impacted tests, and rule analysis are running in the background.",
                        ),
                ),
            RunState::Failed(error) => panel()
                .flex()
                .flex_col()
                .gap_3()
                .child(
                    div()
                        .text_lg()
                        .font_weight(FontWeight::SEMIBOLD)
                        .text_color(rgb(0xfca5a5))
                        .child("Verification failed"),
                )
                .child(div().text_sm().child(error.clone())),
            RunState::Finished(snapshot) => render_snapshot(snapshot),
        };

        div()
            .size_full()
            .id("desktop-root")
            .bg(rgb(0x0f172a))
            .text_color(rgb(0xe2e8f0))
            .overflow_scroll()
            .child(
                div()
                    .flex()
                    .flex_col()
                    .gap_4()
                    .p_4()
                    .child(
                        div()
                            .flex()
                            .flex_col()
                            .gap_1()
                            .child(
                                div()
                                    .text_2xl()
                                    .font_weight(FontWeight::SEMIBOLD)
                                    .child("VeriPatch"),
                            )
                            .child(
                                div()
                                    .text_sm()
                                    .text_color(rgb(0x94a3b8))
                                    .child(
                                        "Desktop verification shell for local changes, clipboard diffs, and patch files.",
                                    ),
                            ),
                    )
                    .child(
                        div()
                            .flex()
                            .gap_4()
                            .flex_col()
                            .child(
                                panel()
                                    .flex()
                                    .flex_col()
                                    .gap_4()
                                    .child(
                                        div()
                                            .flex()
                                            .flex_col()
                                            .gap_2()
                                            .child(
                                                div()
                                                    .text_sm()
                                                    .text_color(rgb(0x94a3b8))
                                                    .child("Repository"),
                                            )
                                            .child(
                                                div()
                                                    .text_sm()
                                                    .font_weight(FontWeight::MEDIUM)
                                                    .child(self.repo_path.display().to_string()),
                                            )
                                            .child(button(
                                                "choose-repo",
                                                "Choose repository",
                                                false,
                                                cx.listener(|this, _, _, cx| this.select_repo(cx)),
                                            )),
                                    )
                                    .child(
                                        div()
                                            .flex()
                                            .flex_col()
                                            .gap_2()
                                            .child(
                                                div()
                                                    .text_sm()
                                                    .text_color(rgb(0x94a3b8))
                                                    .child("Diff source"),
                                            )
                                            .child(
                                                div()
                                                    .flex()
                                                    .gap_2()
                                                    .flex_wrap()
                                                    .child(button(
                                                        "working-tree",
                                                        "Current working tree",
                                                        self.input_source
                                                            == InputSource::CurrentWorkingTree,
                                                        cx.listener(|this, _, _, cx| {
                                                            this.select_input_source(
                                                                InputSource::CurrentWorkingTree,
                                                                cx,
                                                            )
                                                        }),
                                                    ))
                                                    .child(button(
                                                        "clipboard-diff",
                                                        "Clipboard diff",
                                                        self.input_source
                                                            == InputSource::ClipboardDiff,
                                                        cx.listener(|this, _, _, cx| {
                                                            this.select_input_source(
                                                                InputSource::ClipboardDiff,
                                                                cx,
                                                            )
                                                        }),
                                                    ))
                                                    .child(button(
                                                        "patch-file",
                                                        "Patch file",
                                                        self.input_source == InputSource::PatchFile,
                                                        cx.listener(|this, _, _, cx| {
                                                            this.select_input_source(
                                                                InputSource::PatchFile,
                                                                cx,
                                                            )
                                                        }),
                                                    )),
                                            )
                                            .when(
                                                self.input_source == InputSource::ClipboardDiff,
                                                |container| {
                                                    container.child(
                                                        div()
                                                            .flex()
                                                            .flex_col()
                                                            .gap_2()
                                                            .child(button(
                                                                "capture-clipboard",
                                                                "Load diff from clipboard",
                                                                false,
                                                                cx.listener(|this, _, _, cx| {
                                                                    this.capture_clipboard_diff(cx)
                                                                }),
                                                            ))
                                                            .child(
                                                                div()
                                                                    .text_sm()
                                                                    .text_color(rgb(0x94a3b8))
                                                                    .child(clipboard_summary),
                                                            ),
                                                    )
                                                },
                                            )
                                            .when(
                                                self.input_source == InputSource::PatchFile,
                                                |container| {
                                                    container.child(
                                                        div()
                                                            .flex()
                                                            .flex_col()
                                                            .gap_2()
                                                            .child(button(
                                                                "select-patch",
                                                                "Choose patch file",
                                                                false,
                                                                cx.listener(|this, _, _, cx| {
                                                                    this.select_patch_file(cx)
                                                                }),
                                                            ))
                                                            .child(
                                                                div()
                                                                    .text_sm()
                                                                    .text_color(rgb(0x94a3b8))
                                                                    .child(patch_summary),
                                                            ),
                                                    )
                                                },
                                            ),
                                    )
                                    .child(button(
                                        "run-verification",
                                        "Run verification",
                                        true,
                                        cx.listener(|this, _, _, cx| this.start_verification(cx)),
                                    )),
                            )
                            .child(result_panel),
                    ),
            )
    }
}
