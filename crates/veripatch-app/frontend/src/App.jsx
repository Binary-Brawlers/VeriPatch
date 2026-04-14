import React from "react";
import { invoke } from "@tauri-apps/api/core";
import { DEFAULT_UI_SETTINGS } from "./constants/ui";
import { SettingsView } from "./components/SettingsView";
import { Sidebar } from "./components/Sidebar";
import { WorkspaceView } from "./components/WorkspaceView";
import { LogoIcon } from "./components/icons";
import {
  getActiveProject,
  getCurrentRunIndex,
  getDisplayedRunState,
  loadUiSettings,
  saveUiSettings,
} from "./utils/app";

export function App() {
  const [frontendState, setFrontendState] = React.useState(null);
  const [activeView, setActiveView] = React.useState("workspace");
  const [activeSettingsCategory, setActiveSettingsCategory] = React.useState("general");
  const [uiSettings, setUiSettings] = React.useState(loadUiSettings);
  const [historyPageByProjectId, setHistoryPageByProjectId] = React.useState({});
  const [activeRunIndexByProjectId, setActiveRunIndexByProjectId] = React.useState({});
  const [exportStatusByProjectId, setExportStatusByProjectId] = React.useState({});

  React.useEffect(() => {
    let canceled = false;

    async function init() {
      try {
        const nextState = await invoke("get_state");
        if (!canceled) {
          setFrontendState(nextState);
        }
      } catch (error) {
        console.error("get_state:", error);
      }
    }

    init();
    return () => {
      canceled = true;
    };
  }, []);

  React.useEffect(() => {
    if (!frontendState) {
      return;
    }

    document.documentElement.setAttribute("data-theme", frontendState.theme);
  }, [frontendState]);

  React.useEffect(() => {
    document.body.classList.toggle("wrap-diff-lines", !!uiSettings.wrapDiffLines);
    document.body.classList.toggle(
      "hide-diff-line-numbers",
      !uiSettings.showDiffLineNumbers,
    );
    saveUiSettings(uiSettings);
  }, [uiSettings]);

  React.useEffect(() => {
    window.addProjectFromMenu = () => {
      void addProject();
    };
    window.setThemeFromMenu = (theme) => {
      void setTheme(theme);
    };
    window.openSettingsFromMenu = () => {
      openSettings();
    };

    return () => {
      delete window.addProjectFromMenu;
      delete window.setThemeFromMenu;
      delete window.openSettingsFromMenu;
    };
  });

  const activeProject = getActiveProject(frontendState);
  const activeRunIndex = getCurrentRunIndex(activeProject, activeRunIndexByProjectId);
  const displayedRunState = getDisplayedRunState(activeProject, activeRunIndex);
  const exportStatus =
    (activeProject && exportStatusByProjectId[activeProject.id]) ||
    (displayedRunState?.kind === "finished" ? "Ready to export" : "");

  function updateActiveProject(mutator) {
    setFrontendState((current) => {
      if (!current?.active_project_id) {
        return current;
      }

      return {
        ...current,
        projects: current.projects.map((project) => {
          if (project.id !== current.active_project_id) {
            return project;
          }

          return mutator({ ...project });
        }),
      };
    });
  }

  async function syncState() {
    const nextState = await invoke("get_state");
    setFrontendState(nextState);
    return nextState;
  }

  function openSettings() {
    setActiveView("settings");
  }

  function openWorkspace() {
    setActiveView("workspace");
  }

  async function addProject() {
    try {
      setActiveView("workspace");
      setFrontendState(await invoke("add_project"));
    } catch (error) {
      console.error("add_project:", error);
    }
  }

  async function removeProject(projectId) {
    try {
      setFrontendState(await invoke("remove_project", { projectId }));
      setHistoryPageByProjectId((current) => {
        const next = { ...current };
        delete next[projectId];
        return next;
      });
      setActiveRunIndexByProjectId((current) => {
        const next = { ...current };
        delete next[projectId];
        return next;
      });
      setExportStatusByProjectId((current) => {
        const next = { ...current };
        delete next[projectId];
        return next;
      });
    } catch (error) {
      console.error("remove_project:", error);
    }
  }

  async function selectProject(projectId) {
    try {
      setActiveView("workspace");
      setFrontendState(await invoke("select_project", { projectId }));
    } catch (error) {
      console.error("select_project:", error);
    }
  }

  async function setTheme(theme) {
    try {
      setFrontendState(await invoke("set_theme", { theme }));
    } catch (error) {
      console.error("set_theme:", error);
    }
  }

  async function setSource(source) {
    try {
      let nextState = await invoke("set_input_source", { source });
      if (source === "pull_request") {
        nextState = await invoke("refresh_pull_requests");
      }
      setFrontendState(nextState);
    } catch (error) {
      try {
        await syncState();
      } catch (syncError) {
        console.error("get_state:", syncError);
      }
      console.error("set_input_source:", error);
    }
  }

  async function captureClipboard() {
    try {
      const diffText = await navigator.clipboard.readText();
      setFrontendState(await invoke("set_clipboard_diff", { diffText }));
    } catch (error) {
      console.error("set_clipboard_diff:", error);
    }
  }

  async function pickPatchFile() {
    try {
      setFrontendState(await invoke("pick_patch_file"));
    } catch (error) {
      console.error("pick_patch_file:", error);
    }
  }

  async function refreshPullRequests() {
    updateActiveProject((project) => ({
      ...project,
      pull_request_busy: true,
      pull_request_error: null,
      pull_request_message: "Refreshing pull requests…",
    }));

    try {
      setFrontendState(await invoke("refresh_pull_requests"));
    } catch (error) {
      try {
        await syncState();
      } catch (syncError) {
        console.error("get_state:", syncError);
        updateActiveProject((project) => ({
          ...project,
          pull_request_busy: false,
          pull_request_error: String(error),
          pull_request_message: null,
        }));
      }
      console.error("refresh_pull_requests:", error);
    }
  }

  async function selectPullRequest(number) {
    try {
      setFrontendState(await invoke("select_pull_request", { number }));
    } catch (error) {
      console.error("select_pull_request:", error);
    }
  }

  async function mergeSelectedPullRequest() {
    const number = activeProject?.selected_pull_request_number;
    if (!number || !window.confirm(`Merge pull request #${number}?`)) {
      return;
    }

    try {
      setFrontendState(await invoke("merge_selected_pull_request"));
    } catch (error) {
      try {
        await syncState();
      } catch (syncError) {
        console.error("get_state:", syncError);
      }
      console.error("merge_selected_pull_request:", error);
    }
  }

  async function closeSelectedPullRequest() {
    const number = activeProject?.selected_pull_request_number;
    if (!number || !window.confirm(`Close pull request #${number}?`)) {
      return;
    }

    try {
      setFrontendState(await invoke("close_selected_pull_request"));
    } catch (error) {
      try {
        await syncState();
      } catch (syncError) {
        console.error("get_state:", syncError);
      }
      console.error("close_selected_pull_request:", error);
    }
  }

  async function runVerification() {
    if (activeProject) {
      updateActiveProject((project) => ({
        ...project,
        run_state: { kind: "running" },
      }));
    }

    try {
      const nextState = await invoke("run_verification");
      setFrontendState(nextState);
      const refreshedProject = getActiveProject(nextState);
      if (refreshedProject) {
        setHistoryPageByProjectId((current) => ({
          ...current,
          [refreshedProject.id]: 0,
        }));
        setActiveRunIndexByProjectId((current) => ({
          ...current,
          [refreshedProject.id]: 0,
        }));
      }
    } catch (error) {
      updateActiveProject((project) => ({
        ...project,
        run_state: { kind: "failed", data: String(error) },
      }));
    }
  }

  async function exportMarkdownReport() {
    const snapshot = displayedRunState?.kind === "finished" ? displayedRunState.data : null;
    const projectId = activeProject?.id;
    if (!snapshot || !projectId) {
      return;
    }

    try {
      const savedPath = await invoke("export_markdown_report", { snapshot });
      const label = `Saved to ${savedPath.split("/").pop() || savedPath}`;
      setExportStatusByProjectId((current) => ({
        ...current,
        [projectId]: label,
      }));

      window.setTimeout(() => {
        setExportStatusByProjectId((current) => {
          const next = { ...current };
          delete next[projectId];
          return next;
        });
      }, 2500);
    } catch (error) {
      console.error("export_markdown_report:", error);
    }
  }

  function setHistoryPage(projectId, page) {
    setHistoryPageByProjectId((current) => ({
      ...current,
      [projectId]: Math.max(0, page),
    }));
  }

  function openRunFromHistory(index) {
    if (!activeProject?.run_history?.[index]) {
      return;
    }

    setActiveRunIndexByProjectId((current) => ({
      ...current,
      [activeProject.id]: index,
    }));
  }

  function updateUiSetting(key, value) {
    setUiSettings((current) => ({
      ...current,
      [key]: value,
    }));
  }

  function resetUiSettings() {
    setUiSettings({ ...DEFAULT_UI_SETTINGS });
    void setTheme("system");
  }

  return (
    <div id="app">
      <Sidebar
        activeSettingsCategory={activeSettingsCategory}
        activeView={activeView}
        frontendState={frontendState}
        onAddProject={addProject}
        onOpenSettings={openSettings}
        onOpenWorkspace={openWorkspace}
        onRemoveProject={removeProject}
        onSelectProject={selectProject}
        onSelectSettingsCategory={setActiveSettingsCategory}
      />

      <main id="main">
        {!frontendState ? (
          <LoadingWorkspace />
        ) : activeView === "settings" ? (
          <SettingsView
            activeSettingsCategory={activeSettingsCategory}
            onReset={resetUiSettings}
            onSetTheme={setTheme}
            onUpdateUiSetting={updateUiSetting}
            theme={frontendState?.theme || "system"}
            uiSettings={uiSettings}
          />
        ) : activeProject ? (
          <WorkspaceView
            activeProject={activeProject}
            activeRunIndex={activeRunIndex}
            displayedRunState={displayedRunState}
            exportStatus={exportStatus}
            historyPageByProjectId={historyPageByProjectId}
            onCaptureClipboard={captureClipboard}
            onClosePullRequest={closeSelectedPullRequest}
            onExportMarkdownReport={exportMarkdownReport}
            onHistoryPageChange={setHistoryPage}
            onMergePullRequest={mergeSelectedPullRequest}
            onOpenRunFromHistory={openRunFromHistory}
            onPickPatchFile={pickPatchFile}
            onRefreshPullRequests={refreshPullRequests}
            onRunVerification={runVerification}
            onSelectPullRequest={selectPullRequest}
            onSetSource={setSource}
            uiSettings={uiSettings}
          />
        ) : (
          <EmptyWorkspace onAddProject={addProject} />
        )}
      </main>
    </div>
  );
}

function LoadingWorkspace() {
  return (
    <div className="main-empty" id="view-empty">
      <LogoIcon width={48} height={48} opacity={0.3} strokeWidth={1.5} />
      <h2>Loading VeriPatch</h2>
      <p>Restoring your desktop state and project context.</p>
    </div>
  );
}

function EmptyWorkspace({ onAddProject }) {
  return (
    <div className="main-empty" id="view-empty">
      <LogoIcon width={48} height={48} opacity={0.3} strokeWidth={1.5} />
      <h2>Welcome to VeriPatch</h2>
      <p>Add a project to start verifying AI-generated code changes.</p>
      <button className="btn btn-primary" id="btn-empty-add" onClick={onAddProject} type="button">
        Add Project
      </button>
    </div>
  );
}
