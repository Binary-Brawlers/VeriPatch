use std::fs;

use chrono::Utc;
use tauri::State;
use veripatch_core::{VerificationInput, VerificationMode, load_local_diff, verify};
use veripatch_report::markdown::render_markdown_with_source;

use super::storage;
use super::types::*;

// ── App-level commands ─────────────────────────────────────────────

#[tauri::command]
pub(crate) fn get_state(state: State<'_, AppState>) -> FrontendState {
    state.to_frontend_state()
}

#[tauri::command]
pub(crate) fn get_run_history(
    project_id: String,
    state: State<'_, AppState>,
) -> Result<Vec<VerificationRunRecord>, String> {
    let projects = state.projects.lock().unwrap();
    let project = projects
        .iter()
        .find(|p| p.id == project_id)
        .ok_or("Project not found")?;
    Ok(project.run_history.clone())
}

#[tauri::command]
pub(crate) fn set_theme(theme: Theme, state: State<'_, AppState>) -> Result<FrontendState, String> {
    *state.theme.lock().unwrap() = theme;
    persist_state(&state)?;
    Ok(state.to_frontend_state())
}

// ── Project management ─────────────────────────────────────────────

#[tauri::command]
pub(crate) async fn add_project(state: State<'_, AppState>) -> Result<FrontendState, String> {
    let picked = tauri::async_runtime::spawn_blocking(|| rfd::FileDialog::new().pick_folder())
        .await
        .map_err(|e| e.to_string())?;

    let path = picked.ok_or("No folder selected")?;
    let name = path
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_else(|| path.display().to_string());

    let id = state.next_project_id();
    let project = ProjectState::new(id.clone(), name, path);

    {
        let mut projects = state.projects.lock().unwrap();
        projects.push(project);
    }
    *state.active_project_id.lock().unwrap() = Some(id);

    persist_state(&state)?;

    Ok(state.to_frontend_state())
}

#[tauri::command]
pub(crate) fn remove_project(
    project_id: String,
    state: State<'_, AppState>,
) -> Result<FrontendState, String> {
    let next_active_project_id = {
        let mut projects = state.projects.lock().unwrap();
        projects.retain(|p| p.id != project_id);
        projects.first().map(|p| p.id.clone())
    };

    {
        let mut active = state.active_project_id.lock().unwrap();
        if active.as_deref() == Some(&project_id) {
            *active = next_active_project_id;
        }
    }

    persist_state(&state)?;

    Ok(state.to_frontend_state())
}

#[tauri::command]
pub(crate) fn select_project(
    project_id: String,
    state: State<'_, AppState>,
) -> Result<FrontendState, String> {
    *state.active_project_id.lock().unwrap() = Some(project_id);
    persist_state(&state)?;
    Ok(state.to_frontend_state())
}

// ── Per-project commands ───────────────────────────────────────────

fn with_active_project<F>(state: &AppState, f: F) -> Result<(), String>
where
    F: FnOnce(&mut ProjectState),
{
    let active_id = state
        .active_project_id
        .lock()
        .unwrap()
        .clone()
        .ok_or("No active project")?;
    let mut projects = state.projects.lock().unwrap();
    let project = projects
        .iter_mut()
        .find(|p| p.id == active_id)
        .ok_or("Active project not found")?;
    f(project);
    Ok(())
}

fn set_active_project_run_state(state: &AppState, run_state: RunState) -> Result<(), String> {
    with_active_project(state, move |project| {
        project.run_state = run_state;
    })
}

fn persist_failed_run_state(state: &AppState, message: String) {
    let _ = set_active_project_run_state(state, RunState::Failed(message));
    let _ = persist_state(state);
}

#[tauri::command]
pub(crate) fn set_input_source(
    source: InputSource,
    state: State<'_, AppState>,
) -> Result<FrontendState, String> {
    with_active_project(&state, |p| {
        p.input_source = source;
    })?;
    persist_state(&state)?;
    Ok(state.to_frontend_state())
}

#[tauri::command]
pub(crate) fn set_clipboard_diff(
    diff_text: String,
    state: State<'_, AppState>,
) -> Result<FrontendState, String> {
    with_active_project(&state, |p| {
        if diff_text.trim().is_empty() {
            p.run_state =
                RunState::Failed("Clipboard is empty or does not contain a unified diff.".into());
        } else {
            p.clipboard_diff = Some(diff_text.clone());
            p.input_source = InputSource::ClipboardDiff;
            p.run_state = RunState::Idle;
        }
    })?;
    persist_state(&state)?;
    Ok(state.to_frontend_state())
}

#[tauri::command]
pub(crate) async fn pick_patch_file(state: State<'_, AppState>) -> Result<FrontendState, String> {
    let repo_path = {
        let active_id = state
            .active_project_id
            .lock()
            .unwrap()
            .clone()
            .ok_or("No active project")?;
        let projects = state.projects.lock().unwrap();
        let project = projects
            .iter()
            .find(|p| p.id == active_id)
            .ok_or("Active project not found")?;
        project.repo_path.clone()
    };

    let picked = tauri::async_runtime::spawn_blocking(move || {
        let mut dialog = rfd::FileDialog::new().add_filter("Patch", &["patch", "diff"]);
        if repo_path.exists() {
            dialog = dialog.set_directory(&repo_path);
        }
        dialog.pick_file()
    })
    .await
    .map_err(|e| e.to_string())?;

    if let Some(path) = picked {
        with_active_project(&state, |p| {
            p.patch_path = Some(path);
            p.input_source = InputSource::PatchFile;
            p.run_state = RunState::Idle;
        })?;
        persist_state(&state)?;
    }

    Ok(state.to_frontend_state())
}

#[tauri::command]
pub(crate) async fn run_verification(state: State<'_, AppState>) -> Result<FrontendState, String> {
    let (repo_path, input_source, clipboard_diff, patch_path) = {
        let active_id = state
            .active_project_id
            .lock()
            .unwrap()
            .clone()
            .ok_or("No active project")?;
        let mut projects = state.projects.lock().unwrap();
        let project = projects
            .iter_mut()
            .find(|p| p.id == active_id)
            .ok_or("Active project not found")?;
        project.run_state = RunState::Running;
        (
            project.repo_path.clone(),
            project.input_source,
            project.clipboard_diff.clone(),
            project.patch_path.clone(),
        )
    };

    persist_state(&state)?;

    let verification_result = async {
        let (diff_text, mode, source_label) = match input_source {
            InputSource::CurrentWorkingTree => {
                let diff = load_local_diff(&repo_path)
                    .await
                    .map_err(|e| format!("{e:#}"))?;
                (
                    diff,
                    VerificationMode::CurrentWorkingTree,
                    "Working tree".to_string(),
                )
            }
            InputSource::ClipboardDiff => {
                let diff = clipboard_diff
                    .ok_or("Load a unified diff from the clipboard before running verification")?;
                (
                    diff,
                    VerificationMode::ApplyPatchToTempClone,
                    "Clipboard diff".to_string(),
                )
            }
            InputSource::PatchFile => {
                let path = patch_path
                    .ok_or("Choose a .patch or .diff file before running verification")?;
                let diff = fs::read_to_string(&path)
                    .map_err(|e| format!("failed to read patch file `{}`: {e}", path.display()))?;
                (
                    diff,
                    VerificationMode::ApplyPatchToTempClone,
                    format!("Patch: {}", path.display()),
                )
            }
        };

        let result = verify(VerificationInput {
            repo_path,
            diff_text,
            mode,
        })
        .await
        .map_err(|e| format!("{e:#}"))?;

        Ok::<_, String>(VerificationSnapshot {
            source_label,
            result,
        })
    }
    .await;

    let snapshot = match verification_result {
        Ok(snapshot) => snapshot,
        Err(message) => {
            persist_failed_run_state(&state, message.clone());
            return Err(message);
        }
    };

    let run_record = VerificationRunRecord {
        run_id: state.next_run_id(),
        ran_at: Utc::now().to_rfc3339(),
        snapshot: snapshot.clone(),
    };

    with_active_project(&state, |p| {
        p.run_state = RunState::Finished(snapshot);
        p.run_history.insert(0, run_record);
    })?;

    persist_state(&state)?;

    Ok(state.to_frontend_state())
}

#[tauri::command]
pub(crate) async fn export_markdown_report(
    snapshot: VerificationSnapshot,
) -> Result<String, String> {
    let markdown = render_markdown_with_source(Some(&snapshot.source_label), &snapshot.result)
        .map_err(|e| format!("failed to render markdown report: {e:#}"))?;
    let default_file_name = default_report_file_name(&snapshot);

    let picked = tauri::async_runtime::spawn_blocking(move || {
        rfd::FileDialog::new()
            .set_file_name(&default_file_name)
            .add_filter("Markdown", &["md"])
            .save_file()
    })
    .await
    .map_err(|e| e.to_string())?;

    let Some(path) = picked else {
        return Err("Export canceled".into());
    };

    fs::write(&path, markdown)
        .map_err(|e| format!("failed to write markdown report to `{}`: {e}", path.display()))?;

    Ok(path.display().to_string())
}

fn persist_state(state: &AppState) -> Result<(), String> {
    storage::persist_state(state).map_err(|e| format!("failed to persist state: {e:#}"))
}

fn default_report_file_name(snapshot: &VerificationSnapshot) -> String {
    let repo_name = snapshot
        .result
        .repo_path
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("veripatch-report");
    let safe_repo_name: String = repo_name
        .chars()
        .map(|ch| {
            if ch.is_ascii_alphanumeric() || ch == '-' || ch == '_' {
                ch
            } else {
                '-'
            }
        })
        .collect();

    if safe_repo_name.is_empty() {
        "veripatch-report.md".to_string()
    } else {
        format!("veripatch-report-{safe_repo_name}.md")
    }
}
