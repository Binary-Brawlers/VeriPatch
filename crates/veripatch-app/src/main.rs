#[cfg(feature = "desktop-ui")]
mod desktop_ui {
    use anyhow::Result;
    use clap::Parser;
    use gpui::{
        Application, Context, FontWeight, Render, Window, WindowOptions, div, prelude::*, rgb,
    };
    use std::path::PathBuf;

    use veripatch_core::{VerificationInput, VerificationMode, load_local_diff, verify};
    use veripatch_report::render_markdown;

    #[derive(Parser, Debug)]
    #[command(name = "veripatch", version, about)]
    struct Cli {
        /// Path to the repository to verify against.
        #[arg(short, long)]
        repo: Option<String>,
    }

    struct DesktopState {
        title: String,
        repo_path: String,
        report: String,
    }

    impl Render for DesktopState {
        fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
            div()
                .size_full()
                .bg(rgb(0x0f172a))
                .text_color(rgb(0xe2e8f0))
                .child(
                    div()
                        .size_full()
                        .flex()
                        .flex_col()
                        .p_3()
                        .child(
                            div()
                                .w_full()
                                .pb_2()
                                .child(
                                    div()
                                        .text_xl()
                                        .font_weight(FontWeight::SEMIBOLD)
                                        .child(self.title.clone()),
                                )
                                .child(div().text_sm().child(self.repo_path.clone())),
                        )
                        .child(
                            div()
                                .w_full()
                                .flex_1()
                                .rounded_lg()
                                .bg(rgb(0x111827))
                                .p_3()
                                .child(self.report.clone()),
                        ),
                )
        }
    }

    pub fn run() -> Result<()> {
        tracing_subscriber::fmt::init();

        let cli = Cli::parse();
        let repo_path = resolve_repo_path(cli.repo)?;
        let app_state = build_desktop_state(repo_path)?;

        Application::with_platform(gpui_platform::current_platform(false)).run(move |cx| {
            cx.open_window(WindowOptions::default(), move |_, cx| cx.new(|_| app_state))
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

    fn build_desktop_state(repo_path: PathBuf) -> Result<DesktopState> {
        let runtime = tokio::runtime::Runtime::new()?;
        let repo_label = repo_path.display().to_string();

        let (title, report) = runtime.block_on(async {
            match load_local_diff(&repo_path).await {
                Ok(diff_text) => match verify(VerificationInput {
                    repo_path: repo_path.clone(),
                    diff_text,
                    mode: VerificationMode::CurrentWorkingTree,
                })
                .await
                {
                    Ok(result) => {
                        let title =
                            format!("VeriPatch: {:?} ({} / 100)", result.verdict, result.score);
                        let report = render_markdown(&result).unwrap_or_else(|error| {
                            format!("Failed to render markdown report: {error:#}")
                        });
                        (title, report)
                    }
                    Err(error) => (
                        "VeriPatch: Verification Failed".to_string(),
                        format!(
                            "Verification failed for `{}`.\n\n{error:#}",
                            repo_path.display()
                        ),
                    ),
                },
                Err(error) => (
                    "VeriPatch: No Verifiable Changes".to_string(),
                    format!(
                        "Could not load local repository changes for `{}`.\n\n{error:#}",
                        repo_path.display()
                    ),
                ),
            }
        });

        Ok(DesktopState {
            title,
            repo_path: repo_label,
            report,
        })
    }
}

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
