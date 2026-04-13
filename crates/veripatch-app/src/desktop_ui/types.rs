use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::Mutex;

// ── Theme ──────────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum Theme {
    Light,
    Dark,
    System,
}

// ── Input source ───────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum InputSource {
    CurrentWorkingTree,
    ClipboardDiff,
    PatchFile,
}

// ── Per-project state ──────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct ProjectEntry {
    pub id: String,
    pub name: String,
    pub repo_path: String,
    pub input_source: InputSource,
    pub clipboard_diff: Option<String>,
    pub patch_path: Option<String>,
    pub run_state: RunState,
}

// ── Run state ──────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind", content = "data")]
#[serde(rename_all = "snake_case")]
pub(crate) enum RunState {
    Idle,
    Running,
    Finished(VerificationSnapshot),
    Failed(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct VerificationSnapshot {
    pub source_label: String,
    pub result: veripatch_core::VerificationResult,
}

// ── Frontend-facing state ──────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct FrontendState {
    pub theme: Theme,
    pub projects: Vec<ProjectEntry>,
    pub active_project_id: Option<String>,
}

// ── Backend mutable state ──────────────────────────────────────────

pub(crate) struct ProjectState {
    pub id: String,
    pub name: String,
    pub repo_path: PathBuf,
    pub input_source: InputSource,
    pub clipboard_diff: Option<String>,
    pub patch_path: Option<PathBuf>,
    pub run_state: RunState,
}

impl ProjectState {
    pub fn new(id: String, name: String, repo_path: PathBuf) -> Self {
        Self {
            id,
            name,
            repo_path,
            input_source: InputSource::CurrentWorkingTree,
            clipboard_diff: None,
            patch_path: None,
            run_state: RunState::Idle,
        }
    }

    pub fn to_entry(&self) -> ProjectEntry {
        ProjectEntry {
            id: self.id.clone(),
            name: self.name.clone(),
            repo_path: self.repo_path.display().to_string(),
            input_source: self.input_source,
            clipboard_diff: self.clipboard_diff.clone(),
            patch_path: self.patch_path.as_ref().map(|p| p.display().to_string()),
            run_state: self.run_state.clone(),
        }
    }
}

pub(crate) struct AppState {
    pub theme: Mutex<Theme>,
    pub projects: Mutex<Vec<ProjectState>>,
    pub active_project_id: Mutex<Option<String>>,
    next_id: Mutex<u64>,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            theme: Mutex::new(Theme::System),
            projects: Mutex::new(Vec::new()),
            active_project_id: Mutex::new(None),
            next_id: Mutex::new(1),
        }
    }
}

impl AppState {
    pub fn next_project_id(&self) -> String {
        let mut counter = self.next_id.lock().unwrap();
        let id = format!("proj-{counter}");
        *counter += 1;
        id
    }

    pub fn to_frontend_state(&self) -> FrontendState {
        let projects = self.projects.lock().unwrap();
        FrontendState {
            theme: *self.theme.lock().unwrap(),
            projects: projects.iter().map(|p| p.to_entry()).collect(),
            active_project_id: self.active_project_id.lock().unwrap().clone(),
        }
    }
}
