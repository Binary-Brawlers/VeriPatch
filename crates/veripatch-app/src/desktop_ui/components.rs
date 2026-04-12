use gpui::{Div, FontWeight, IntoElement, div, prelude::*, px, rgb};

use super::types::VerificationSnapshot;

pub(super) fn panel() -> Div {
    div()
        .rounded_lg()
        .border_1()
        .border_color(rgb(0x1e293b))
        .bg(rgb(0x111827))
        .p_4()
}

pub(super) fn button(
    id: &str,
    label: &str,
    active: bool,
    on_click: impl Fn(&gpui::ClickEvent, &mut gpui::Window, &mut gpui::App) + 'static,
) -> impl IntoElement {
    let background = if active { rgb(0x2563eb) } else { rgb(0x1f2937) };
    let hover = if active { rgb(0x1d4ed8) } else { rgb(0x334155) };
    let border = if active { rgb(0x60a5fa) } else { rgb(0x475569) };

    div()
        .id(id.to_string())
        .px_3()
        .py_2()
        .rounded_md()
        .border_1()
        .border_color(border)
        .bg(background)
        .hover(move |style| style.bg(hover))
        .cursor_pointer()
        .text_sm()
        .font_weight(FontWeight::MEDIUM)
        .on_click(on_click)
        .child(label.to_string())
}

pub(super) fn render_snapshot(snapshot: &VerificationSnapshot) -> Div {
    let result = &snapshot.result;

    panel()
        .flex()
        .flex_col()
        .gap_4()
        .child(
            div()
                .flex()
                .flex_wrap()
                .gap_3()
                .child(metric_card("Verdict", verdict_label(result.verdict)))
                .child(metric_card("Score", format!("{} / 100", result.score)))
                .child(metric_card("Input", snapshot.source_label.clone()))
                .child(metric_card(
                    "Scope",
                    format!(
                        "{} file(s), +{}, -{}",
                        result.diff.files.len(),
                        result.diff.total_additions,
                        result.diff.total_deletions
                    ),
                )),
        )
        .child(
            section("Checks").children(result.checks.iter().map(|check| {
                div()
                    .flex()
                    .flex_col()
                    .gap_1()
                    .rounded_md()
                    .bg(rgb(0x0f172a))
                    .border_1()
                    .border_color(rgb(0x1e293b))
                    .p_3()
                    .child(format!(
                        "[{}] {}",
                        check_status_label(check.status),
                        check.name
                    ))
                    .child(
                        div()
                            .text_sm()
                            .text_color(rgb(0x94a3b8))
                            .child(check.summary.clone()),
                    )
            })),
        )
        .child(
            section("Changed files").children(result.diff.files.iter().map(|file| {
                div().text_sm().child(format!(
                    "{} ({:?}, +{}, -{})",
                    file.display_path(),
                    file.change_type,
                    file.additions,
                    file.deletions
                ))
            })),
        )
        .when(!result.risky_patterns.is_empty(), |container| {
            container.child(
                section("Risky patterns").children(result.risky_patterns.iter().map(|finding| {
                    div().text_sm().child(format!(
                        "[{}] {}{}",
                        severity_label(finding.severity),
                        finding.message,
                        format_location(finding.file_path.as_deref(), finding.line_number)
                    ))
                })),
            )
        })
        .when(!result.assumptions.is_empty(), |container| {
            container.child(
                section("Assumptions").children(result.assumptions.iter().map(|assumption| {
                    div().text_sm().child(format!(
                        "{}{}",
                        assumption.message,
                        format_location(assumption.file_path.as_deref(), assumption.line_number)
                    ))
                })),
            )
        })
        .when(!result.dependency_notes.is_empty(), |container| {
            container.child(
                section("Dependency review").children(
                    result
                        .dependency_notes
                        .iter()
                        .map(|note| div().text_sm().child(note.clone())),
                ),
            )
        })
        .when(!result.warnings.is_empty(), |container| {
            container.child(
                section("Warnings").children(
                    result
                        .warnings
                        .iter()
                        .map(|warning| div().text_sm().child(warning.clone())),
                ),
            )
        })
}

pub(super) fn section(title: &str) -> Div {
    panel().flex().flex_col().gap_2().child(
        div()
            .text_base()
            .font_weight(FontWeight::SEMIBOLD)
            .child(title.to_string()),
    )
}

fn metric_card(label: &str, value: impl Into<String>) -> Div {
    div()
        .min_w(px(180.0))
        .flex()
        .flex_col()
        .gap_1()
        .rounded_md()
        .border_1()
        .border_color(rgb(0x1e293b))
        .bg(rgb(0x0f172a))
        .p_3()
        .child(
            div()
                .text_xs()
                .text_color(rgb(0x94a3b8))
                .child(label.to_string()),
        )
        .child(
            div()
                .text_base()
                .font_weight(FontWeight::SEMIBOLD)
                .child(value.into()),
        )
}

fn verdict_label(verdict: veripatch_core::Verdict) -> &'static str {
    match verdict {
        veripatch_core::Verdict::Safe => "Safe",
        veripatch_core::Verdict::Risky => "Risky",
        veripatch_core::Verdict::Broken => "Broken",
    }
}

fn check_status_label(status: veripatch_runners::runner::CheckStatus) -> &'static str {
    match status {
        veripatch_runners::runner::CheckStatus::Passed => "pass",
        veripatch_runners::runner::CheckStatus::Failed => "fail",
        veripatch_runners::runner::CheckStatus::Skipped => "skip",
    }
}

fn severity_label(severity: veripatch_rules::rule::RiskSeverity) -> &'static str {
    match severity {
        veripatch_rules::rule::RiskSeverity::Low => "low",
        veripatch_rules::rule::RiskSeverity::Medium => "medium",
        veripatch_rules::rule::RiskSeverity::High => "high",
    }
}

fn format_location(file_path: Option<&str>, line_number: Option<usize>) -> String {
    match (file_path, line_number) {
        (Some(file_path), Some(line_number)) => format!(" (`{}:{}`)", file_path, line_number),
        (Some(file_path), None) => format!(" (`{}`)", file_path),
        _ => String::new(),
    }
}
