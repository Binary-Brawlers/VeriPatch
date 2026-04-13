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
    PullRequest,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct PullRequestSummary {
    pub number: u64,
    pub title: String,
    pub author: String,
    pub base_ref_name: String,
    pub head_ref_name: String,
    pub updated_at: String,
    pub is_draft: bool,
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
    pub pull_requests: Vec<PullRequestSummary>,
    pub selected_pull_request_number: Option<u64>,
    pub pull_request_busy: bool,
    pub pull_request_message: Option<String>,
    pub pull_request_error: Option<String>,
    pub run_state: RunState,
    pub run_history: Vec<VerificationRunRecord>,
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct VerificationRunRecord {
    pub run_id: String,
    pub ran_at: String,
    pub snapshot: VerificationSnapshot,
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
    pub pull_requests: Vec<PullRequestSummary>,
    pub selected_pull_request_number: Option<u64>,
    pub pull_request_busy: bool,
    pub pull_request_message: Option<String>,
    pub pull_request_error: Option<String>,
    pub run_state: RunState,
    pub run_history: Vec<VerificationRunRecord>,
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
            pull_requests: Vec::new(),
            selected_pull_request_number: None,
            pull_request_busy: false,
            pull_request_message: None,
            pull_request_error: None,
            run_state: RunState::Idle,
            run_history: Vec::new(),
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
            pull_requests: self.pull_requests.clone(),
            selected_pull_request_number: self.selected_pull_request_number,
            pull_request_busy: self.pull_request_busy,
            pull_request_message: self.pull_request_message.clone(),
            pull_request_error: self.pull_request_error.clone(),
            run_state: self.run_state.clone(),
            run_history: self.run_history.clone(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct PersistedProjectState {
    pub id: String,
    pub name: String,
    pub repo_path: String,
    pub input_source: InputSource,
    pub clipboard_diff: Option<String>,
    pub patch_path: Option<String>,
    #[serde(default)]
    pub selected_pull_request_number: Option<u64>,
    pub run_state: RunState,
    #[serde(default)]
    pub run_history: Vec<VerificationRunRecord>,
}

impl From<&ProjectState> for PersistedProjectState {
    fn from(value: &ProjectState) -> Self {
        Self {
            id: value.id.clone(),
            name: value.name.clone(),
            repo_path: value.repo_path.display().to_string(),
            input_source: value.input_source,
            clipboard_diff: value.clipboard_diff.clone(),
            patch_path: value.patch_path.as_ref().map(|p| p.display().to_string()),
            selected_pull_request_number: value.selected_pull_request_number,
            run_state: value.run_state.clone(),
            run_history: value.run_history.clone(),
        }
    }
}

impl From<PersistedProjectState> for ProjectState {
    fn from(value: PersistedProjectState) -> Self {
        Self {
            id: value.id,
            name: value.name,
            repo_path: PathBuf::from(value.repo_path),
            input_source: value.input_source,
            clipboard_diff: value.clipboard_diff,
            patch_path: value.patch_path.map(PathBuf::from),
            pull_requests: Vec::new(),
            selected_pull_request_number: value.selected_pull_request_number,
            pull_request_busy: false,
            pull_request_message: None,
            pull_request_error: None,
            run_state: value.run_state,
            run_history: value.run_history,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct PersistedAppState {
    pub theme: Theme,
    pub projects: Vec<PersistedProjectState>,
    pub active_project_id: Option<String>,
    pub next_project_id: u64,
    pub next_run_id: u64,
}

impl Default for PersistedAppState {
    fn default() -> Self {
        Self {
            theme: Theme::System,
            projects: Vec::new(),
            active_project_id: None,
            next_project_id: 1,
            next_run_id: 1,
        }
    }
}

pub(crate) struct AppState {
    pub theme: Mutex<Theme>,
    pub projects: Mutex<Vec<ProjectState>>,
    pub active_project_id: Mutex<Option<String>>,
    next_project_id: Mutex<u64>,
    next_run_id: Mutex<u64>,
    pub storage_path: PathBuf,
}

impl AppState {
    pub fn from_persisted(storage_path: PathBuf, persisted: PersistedAppState) -> Self {
        Self {
            theme: Mutex::new(persisted.theme),
            projects: Mutex::new(
                persisted
                    .projects
                    .into_iter()
                    .map(ProjectState::from)
                    .collect(),
            ),
            active_project_id: Mutex::new(persisted.active_project_id),
            next_project_id: Mutex::new(persisted.next_project_id.max(1)),
            next_run_id: Mutex::new(persisted.next_run_id.max(1)),
            storage_path,
        }
    }

    pub fn next_project_id(&self) -> String {
        let mut counter = self.next_project_id.lock().unwrap();
        let id = format!("proj-{counter}");
        *counter += 1;
        id
    }

    pub fn next_run_id(&self) -> String {
        let mut counter = self.next_run_id.lock().unwrap();
        let id = format!("run-{counter}");
        *counter += 1;
        id
    }

    pub fn to_persisted_state(&self) -> PersistedAppState {
        let projects = self.projects.lock().unwrap();
        PersistedAppState {
            theme: *self.theme.lock().unwrap(),
            projects: projects.iter().map(PersistedProjectState::from).collect(),
            active_project_id: self.active_project_id.lock().unwrap().clone(),
            next_project_id: *self.next_project_id.lock().unwrap(),
            next_run_id: *self.next_run_id.lock().unwrap(),
        }
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
