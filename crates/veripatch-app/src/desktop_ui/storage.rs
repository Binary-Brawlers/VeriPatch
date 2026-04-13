use std::fs;
use std::path::{Path, PathBuf};

use anyhow::Context;
use tauri::Manager;

use super::types::{AppState, PersistedAppState};

const STATE_FILE_NAME: &str = "desktop_state.json";

pub(crate) fn load_or_initialize_state(app: &tauri::AppHandle) -> anyhow::Result<AppState> {
    let storage_path = resolve_storage_path(app)?;
    let persisted = load_from_disk(&storage_path).unwrap_or_else(|err| {
        tracing::warn!("failed to load persisted desktop state: {err:#}");
        PersistedAppState::default()
    });

    Ok(AppState::from_persisted(storage_path, persisted))
}

pub(crate) fn persist_state(state: &AppState) -> anyhow::Result<()> {
    let persisted = state.to_persisted_state();
    save_to_disk(&state.storage_path, &persisted)
}

fn resolve_storage_path(app: &tauri::AppHandle) -> anyhow::Result<PathBuf> {
    let app_data_dir = app
        .path()
        .app_data_dir()
        .context("failed to resolve app data directory")?;

    fs::create_dir_all(&app_data_dir).with_context(|| {
        format!(
            "failed to create app data directory `{}`",
            app_data_dir.display()
        )
    })?;

    Ok(app_data_dir.join(STATE_FILE_NAME))
}

fn load_from_disk(path: &Path) -> anyhow::Result<PersistedAppState> {
    if !path.exists() {
        return Ok(PersistedAppState::default());
    }

    let text = fs::read_to_string(path)
        .with_context(|| format!("failed to read desktop state file `{}`", path.display()))?;

    serde_json::from_str(&text)
        .with_context(|| format!("failed to parse desktop state file `{}`", path.display()))
}

fn save_to_disk(path: &Path, state: &PersistedAppState) -> anyhow::Result<()> {
    let parent = path.parent().context("desktop state path missing parent")?;
    fs::create_dir_all(parent).with_context(|| {
        format!(
            "failed to create desktop state parent directory `{}`",
            parent.display()
        )
    })?;

    let json = serde_json::to_string_pretty(state).context("failed to serialize desktop state")?;

    // Write to a sibling temp file, then atomically rename so crashes don't corrupt state.
    let temp_path = path.with_extension("json.tmp");
    fs::write(&temp_path, json).with_context(|| {
        format!(
            "failed to write temporary desktop state `{}`",
            temp_path.display()
        )
    })?;
    fs::rename(&temp_path, path).with_context(|| {
        format!(
            "failed to commit desktop state from `{}` to `{}`",
            temp_path.display(),
            path.display()
        )
    })?;

    Ok(())
}
