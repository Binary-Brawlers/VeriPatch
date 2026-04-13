const { invoke } = window.__TAURI__.core;

let state = null;
const HISTORY_PAGE_SIZE = 10;
const historyPageByProjectId = {};
const activeRunIndexByProjectId = {};
const SETTINGS_STORAGE_KEY = "veripatch.ui.settings.v1";
const defaultUiSettings = {
  timeFormat: "system",
  wrapDiffLines: false,
  showDiffLineNumbers: true,
};
let activeView = "workspace";
let activeSettingsCategory = "general";
let uiSettings = loadUiSettings();

// ─── Init ──────────────────────────────────────────────────────

async function init() {
  try {
    state = await invoke("get_state");
    render();
  } catch (e) {
    console.error("init:", e);
  }

  // Sidebar buttons
  document.getElementById("btn-add-project").addEventListener("click", addProject);
  document.getElementById("btn-add-first").addEventListener("click", addProject);
  document.getElementById("btn-empty-add").addEventListener("click", addProject);

  // Toolbar
  document.getElementById("btn-run").addEventListener("click", runVerification);

  // Source segmented control
  document.querySelectorAll(".seg-btn").forEach((btn) => {
    btn.addEventListener("click", () => setSource(btn.dataset.source));
  });

  // Clipboard / patch buttons
  document.getElementById("btn-clipboard").addEventListener("click", captureClipboard);
  document.getElementById("btn-patch").addEventListener("click", pickPatchFile);

  // Settings
  document.getElementById("btn-open-settings").addEventListener("click", openSettings);
  document.getElementById("btn-close-settings").addEventListener("click", openWorkspace);
  document.getElementById("btn-settings-reset").addEventListener("click", resetUiSettings);
  document.getElementById("setting-theme-select").addEventListener("change", (e) => {
    setTheme(e.target.value);
  });
  document.getElementById("setting-time-format").addEventListener("change", (e) => {
    updateUiSetting("timeFormat", e.target.value);
  });
  document.getElementById("setting-wrap-diff-lines").addEventListener("change", (e) => {
    updateUiSetting("wrapDiffLines", e.target.checked);
  });
  document.getElementById("setting-show-diff-line-numbers").addEventListener("change", (e) => {
    updateUiSetting("showDiffLineNumbers", e.target.checked);
  });
  document.querySelectorAll(".settings-nav-item").forEach((btn) => {
    btn.addEventListener("click", () => setSettingsCategory(btn.dataset.settingsCategory));
  });
}

// ─── Menu bridge (called from Rust) ────────────────────────────

window.addProjectFromMenu = addProject;
window.setThemeFromMenu = function (t) { setTheme(t); };
window.openSettingsFromMenu = openSettings;

// ─── Render ────────────────────────────────────────────────────

function render() {
  if (!state) return;

  applyTheme(state.theme);
  applyUiSettings();
  renderSidebar();

  if (activeView === "settings") {
    document.getElementById("view-empty").style.display = "none";
    document.getElementById("view-project").style.display = "none";
    document.getElementById("view-settings").style.display = "flex";
    renderSettings();
    return;
  }

  document.getElementById("view-settings").style.display = "none";

  const activeProject = getActiveProject();
  if (!activeProject) {
    document.getElementById("view-empty").style.display = "flex";
    document.getElementById("view-project").style.display = "none";
    return;
  }

  document.getElementById("view-empty").style.display = "none";
  document.getElementById("view-project").style.display = "flex";
  document.getElementById("view-project").style.flexDirection = "column";
  document.getElementById("view-project").style.flex = "1";

  renderToolbar(activeProject);
  renderConfigBar(activeProject);
  renderResults(activeProject);
}

function applyTheme(theme) {
  document.documentElement.setAttribute("data-theme", theme);
  const themeSelect = document.getElementById("setting-theme-select");
  if (themeSelect) themeSelect.value = theme;
}

function renderSidebar() {
  const list = document.getElementById("project-list");
  const empty = document.getElementById("no-projects");
  const projectsSection = document.getElementById("sidebar-projects-section");
  const settingsSection = document.getElementById("sidebar-settings-section");
  const openSettingsBtn = document.getElementById("btn-open-settings");
  const closeSettingsBtn = document.getElementById("btn-close-settings");

  if (activeView === "settings") {
    projectsSection.style.display = "none";
    settingsSection.style.display = "block";
    openSettingsBtn.style.display = "none";
    closeSettingsBtn.style.display = "inline-flex";
    document.querySelectorAll(".settings-nav-item").forEach((btn) => {
      btn.classList.toggle("active", btn.dataset.settingsCategory === activeSettingsCategory);
    });
    return;
  }

  projectsSection.style.display = "block";
  settingsSection.style.display = "none";
  openSettingsBtn.style.display = "inline-flex";
  closeSettingsBtn.style.display = "none";

  if (state.projects.length === 0) {
    list.innerHTML = "";
    empty.style.display = "flex";
    return;
  }

  empty.style.display = "none";
  list.innerHTML = state.projects
    .map((p) => {
      const active = p.id === state.active_project_id ? "active" : "";
      return `<li class="project-item ${active}" data-id="${esc(p.id)}">
        <span class="project-label">${esc(p.name)}</span>
        <button class="remove-btn" data-id="${esc(p.id)}" title="Remove project">
          <svg viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" stroke-width="2"><line x1="18" y1="6" x2="6" y2="18"/><line x1="6" y1="6" x2="18" y2="18"/></svg>
        </button>
      </li>`;
    })
    .join("");

  list.querySelectorAll(".project-item").forEach((el) => {
    el.addEventListener("click", (e) => {
      if (e.target.closest(".remove-btn")) return;
      selectProject(el.dataset.id);
    });
  });

  list.querySelectorAll(".remove-btn").forEach((btn) => {
    btn.addEventListener("click", (e) => {
      e.stopPropagation();
      removeProject(btn.dataset.id);
    });
  });
}

function renderToolbar(project) {
  document.getElementById("project-name").textContent = project.name;
  document.getElementById("project-path").textContent = project.repo_path;
}

function renderConfigBar(project) {
  const src = project.input_source;
  document.querySelectorAll(".seg-btn").forEach((btn) => {
    btn.classList.toggle("active", btn.dataset.source === src);
  });

  document.getElementById("clipboard-actions").style.display =
    src === "clipboard_diff" ? "flex" : "none";
  document.getElementById("patch-actions").style.display =
    src === "patch_file" ? "flex" : "none";

  if (project.clipboard_diff) {
    const n = project.clipboard_diff.split("\n").length;
    document.getElementById("clipboard-hint").textContent = `${n} lines loaded`;
  } else {
    document.getElementById("clipboard-hint").textContent = "No diff loaded";
  }

  document.getElementById("patch-hint").textContent =
    project.patch_path || "No file selected";
}

function renderResults(project) {
  const runState = project.run_state;
  const runHistory = project.run_history || [];
  const resultsArea = document.getElementById("results-area");
  const ids = ["result-idle", "result-running", "result-failed", "result-finished"];
  ids.forEach((id) => (document.getElementById(id).style.display = "none"));
  resultsArea.classList.toggle("has-history", runHistory.length > 0);

  switch (runState.kind) {
    case "idle":
      document.getElementById("result-idle").style.display = "flex";
      break;
    case "running":
      document.getElementById("result-running").style.display = "flex";
      break;
    case "failed":
      document.getElementById("result-failed").style.display = "flex";
      document.getElementById("error-message").textContent = runState.data;
      break;
    case "finished":
      document.getElementById("result-finished").style.display = "block";
      renderSnapshot(runState.data);
      break;
  }

  renderRunHistory(project, getCurrentRunIndex(project));
}

function renderRunHistory(project, activeRunIndex = null) {
  const runHistory = project.run_history || [];
  const section = document.getElementById("section-history");
  if (!runHistory || runHistory.length === 0) {
    section.innerHTML = "";
    return;
  }

  const totalPages = Math.max(1, Math.ceil(runHistory.length / HISTORY_PAGE_SIZE));
  const currentPage = Math.min(historyPageByProjectId[project.id] || 0, totalPages - 1);
  historyPageByProjectId[project.id] = currentPage;

  const start = currentPage * HISTORY_PAGE_SIZE;
  const end = Math.min(start + HISTORY_PAGE_SIZE, runHistory.length);
  const pageItems = runHistory.slice(start, end);

  section.innerHTML = `
    <details class="collapsible-section" open>
      <summary class="section-header">
        <svg class="chevron-icon" viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" stroke-width="2"><polyline points="6 9 12 15 18 9"/></svg>
        Run History<span class="count-badge">${runHistory.length}</span>
      </summary>
      <div class="section-content">
        ${pageItems.map((entry, pageIndex) => {
          const index = start + pageIndex;
          const verdict = entry.snapshot?.result?.verdict || "UNKNOWN";
          const verdictClass = `verdict-${String(verdict).toLowerCase()}`;
          const isActive = index === activeRunIndex;
          return `<button type="button" class="history-item ${isActive ? "active" : ""}" data-run-index="${index}" aria-pressed="${isActive}">
            <div class="history-main">
              <div class="history-head">
                <span class="history-run-id">${esc(entry.run_id || `run-${index + 1}`)}</span>
                <span class="history-time">${esc(formatRunTimestamp(entry.ran_at))}</span>
              </div>
              <div class="history-meta">
                <span class="history-source">${esc(entry.snapshot?.source_label || "Unknown source")}</span>
                <span class="history-verdict ${verdictClass}">${esc(verdict)}</span>
                <span class="history-score">${entry.snapshot?.result?.score ?? "-"}/100</span>
              </div>
            </div>
            <span class="history-item-action">${isActive ? "Viewing" : "Open"}</span>
          </button>`;
        }).join("")}
        ${totalPages > 1 ? `<div class="history-pagination">
          <div class="history-page-status">Showing ${start + 1}-${end} of ${runHistory.length}</div>
          <div class="history-page-actions">
            <button class="btn btn-secondary btn-sm history-page-btn" data-history-page="prev" ${currentPage === 0 ? "disabled" : ""}>Previous</button>
            <span class="history-page-indicator">Page ${currentPage + 1} / ${totalPages}</span>
            <button class="btn btn-secondary btn-sm history-page-btn" data-history-page="next" ${currentPage >= totalPages - 1 ? "disabled" : ""}>Next</button>
          </div>
        </div>` : ""}
      </div>
    </details>
  `;

  section.querySelectorAll(".history-item").forEach((item) => {
    item.addEventListener("click", (e) => {
      e.preventDefault();
      const idx = Number(item.dataset.runIndex);
      openRunFromHistory(idx);
    });
  });

  section.querySelectorAll(".history-page-btn").forEach((btn) => {
    btn.addEventListener("click", (e) => {
      e.preventDefault();
      const delta = btn.dataset.historyPage === "next" ? 1 : -1;
      setHistoryPage(project.id, currentPage + delta);
    });
  });
}

function openRunFromHistory(index) {
  const active = getActiveProject();
  if (!active || !active.run_history || !active.run_history[index]) return;
  const entry = active.run_history[index];
  activeRunIndexByProjectId[active.id] = index;
  active.run_state = { kind: "finished", data: entry.snapshot };
  render();
}

function formatRunTimestamp(value) {
  if (!value) return "Unknown time";
  const d = new Date(value);
  if (Number.isNaN(d.getTime())) return value;
  const options = {
    year: "numeric",
    month: "short",
    day: "numeric",
    hour: "2-digit",
    minute: "2-digit",
  };

  if (uiSettings.timeFormat === "24h") {
    options.hour12 = false;
  } else if (uiSettings.timeFormat === "12h") {
    options.hour12 = true;
  }

  return d.toLocaleString(undefined, options);
}

function renderSnapshot(snapshot) {
  const r = snapshot.result;
  const verdictClass = `verdict-${r.verdict.toLowerCase()}`;
  const diffLookup = buildDiffLookup(r.diff.files || []);

  document.getElementById("metrics").innerHTML = [
    `<div class="metric-card"><span class="label">Verdict</span><span class="val ${verdictClass}">${esc(r.verdict)}</span></div>`,
    `<div class="metric-card"><span class="label">Score</span><span class="val">${r.score}<span style="font-size:12px;font-weight:400;color:var(--text-tertiary)"> / 100</span></span></div>`,
    `<div class="metric-card"><span class="label">Source</span><span class="val" style="font-size:14px">${esc(snapshot.source_label)}</span></div>`,
    `<div class="metric-card"><span class="label">Scope</span><span class="val" style="font-size:14px">${r.diff.files.length} files <span style="font-size:12px;font-weight:400;color:var(--text-tertiary)">+${r.diff.total_additions} / -${r.diff.total_deletions}</span></span></div>`,
  ].join("");

  // Checks
  renderSection("section-checks", "Checks", r.checks, (c) => {
    const st = statusKey(c.status);
    const icon = st === "pass" ? "\u2713" : st === "fail" ? "\u2717" : "\u2014";
    return `<div class="check-item">
      <div class="check-status ${st}">${icon}</div>
      <div class="check-info">
        <div class="check-name">${esc(c.name)}</div>
        <div class="check-summary">${esc(c.summary)}</div>
      </div>
    </div>`;
  });

  // Files
  renderSection("section-files", "Changed Files", r.diff.files, (f, i) => {
    const ct = (f.change_type || "Modified").toLowerCase();
    const filePath = f.new_path || f.old_path || "<unknown>";
    const fileName = filePath.split("/").pop();
    const dirPath = filePath.includes("/") ? filePath.substring(0, filePath.lastIndexOf("/") + 1) : "";
    const hasHunks = f.hunks && f.hunks.length > 0;
    const diffHtml = hasHunks
      ? renderDiffPreview(f.hunks)
      : '<div class="diff-preview"><div class="diff-empty">No diff content available</div></div>';
    return `<details class="file-entry" id="file-entry-${i}">
      <summary class="file-item">
        <svg class="file-chevron" viewBox="0 0 24 24" width="12" height="12" fill="none" stroke="currentColor" stroke-width="2"><polyline points="6 9 12 15 18 9"/></svg>
        <span class="change-type ${ct}">${esc(f.change_type || "Mod")}</span>
        <span class="file-path"><span class="file-dir">${esc(dirPath)}</span><span class="file-name">${esc(fileName)}</span></span>
        <span class="diff-stat">+${f.additions} / -${f.deletions}</span>
      </summary>
      ${diffHtml}
    </details>`;
  });

  // Risky patterns
  renderSection("section-risky", "Risky Patterns", r.risky_patterns, (f) => {
    const sev = (f.severity || "low").toLowerCase();
    return renderFinding({
      badge: `<span class="severity severity-${sev}">${esc(f.severity)}</span>`,
      message: f.message,
      filePath: f.file_path,
      lineNumber: f.line_number,
      snippet: renderFindingSnippet(diffLookup, f.file_path, f.line_number),
    });
  });

  // Assumptions
  renderSection("section-assumptions", "Assumptions", r.assumptions, (a) =>
    renderFinding({
      message: a.message,
      filePath: a.file_path,
      lineNumber: a.line_number,
      snippet: renderFindingSnippet(diffLookup, a.file_path, a.line_number),
    })
  );

  // Dependencies
  renderSection("section-deps", "Dependencies", r.dependency_notes, (n) =>
    `<div class="finding-item">${esc(n)}</div>`
  );

  // Warnings
  renderSection("section-warnings", "Warnings", r.warnings, (w) =>
    `<div class="finding-item">${esc(w)}</div>`
  );
}

function renderSection(containerId, title, items, renderItem) {
  const el = document.getElementById(containerId);
  if (!items || items.length === 0) {
    el.innerHTML = "";
    return;
  }
  el.innerHTML = `
    <details class="collapsible-section" open>
      <summary class="section-header">
        <svg class="chevron-icon" viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" stroke-width="2"><polyline points="6 9 12 15 18 9"/></svg>
        ${esc(title)}<span class="count-badge">${items.length}</span>
      </summary>
      <div class="section-content">
        ${items.map(renderItem).join("")}
      </div>
    </details>
  `;
}

// ─── Helpers ───────────────────────────────────────────────────

function setHistoryPage(projectId, page) {
  historyPageByProjectId[projectId] = Math.max(0, page);
  render();
}

function getCurrentRunIndex(project) {
  const runHistory = project.run_history || [];
  if (project.run_state?.kind !== "finished" || runHistory.length === 0) {
    return null;
  }

  const storedIndex = activeRunIndexByProjectId[project.id];
  if (Number.isInteger(storedIndex) && storedIndex >= 0 && storedIndex < runHistory.length) {
    return storedIndex;
  }

  const snapshot = project.run_state.data;
  const snapshotIndex = runHistory.findIndex((entry) => entry.snapshot === snapshot);
  if (snapshotIndex >= 0) {
    activeRunIndexByProjectId[project.id] = snapshotIndex;
    return snapshotIndex;
  }

  activeRunIndexByProjectId[project.id] = 0;
  return 0;
}

function getActiveProject() {
  if (!state || !state.active_project_id) return null;
  return state.projects.find((p) => p.id === state.active_project_id) || null;
}

function renderDiffPreview(hunks, highlightedLineNumber = null) {
  return `<div class="diff-preview">${(hunks || []).map((h) => renderDiffHunk(h, highlightedLineNumber)).join("")}</div>`;
}

function renderDiffHunk(hunk, highlightedLineNumber = null) {
  return `<div class="diff-hunk"><div class="diff-hunk-header">${esc(hunk.header)}</div>${(hunk.lines || []).map((line) => renderDiffLine(line, highlightedLineNumber)).join("")}</div>`;
}

function renderDiffLine(line, highlightedLineNumber = null) {
  const cls = line.kind === "Addition" ? "diff-add" : line.kind === "Deletion" ? "diff-del" : "diff-ctx";
  const prefix = line.kind === "Addition" ? "+" : line.kind === "Deletion" ? "-" : " ";
  const visibleLineNumber = line.kind === "Deletion"
    ? line.old_line_number
    : line.new_line_number;
  const ln = visibleLineNumber != null ? String(visibleLineNumber).padStart(4) : "    ";
  const highlightClass = highlightedLineNumber != null && visibleLineNumber === highlightedLineNumber
    ? " diff-focus"
    : "";

  return `<div class="diff-line ${cls}${highlightClass}"><span class="diff-ln">${ln}</span><span class="diff-prefix">${prefix}</span><span class="diff-text">${esc(line.content)}</span></div>`;
}

function buildDiffLookup(files) {
  const lookup = new Map();

  (files || []).forEach((file) => {
    const filePath = file.new_path || file.old_path;
    if (!filePath) return;
    lookup.set(filePath, file);
  });

  return lookup;
}

function renderFinding({ badge = "", message, filePath, lineNumber, snippet = "" }) {
  const content = snippet || '<div class="diff-preview"><div class="diff-empty">No diff content available</div></div>';
  return `<details class="finding-item">
    <summary class="finding-summary">
      <svg class="finding-chevron" viewBox="0 0 24 24" width="12" height="12" fill="none" stroke="currentColor" stroke-width="2"><polyline points="6 9 12 15 18 9"/></svg>
      <div class="finding-copy">
        ${badge}${esc(message)}
        ${loc(filePath, lineNumber)}
      </div>
    </summary>
    ${content}
  </details>`;
}

function renderFindingSnippet(diffLookup, filePath, lineNumber) {
  if (!filePath) return "";

  const file = diffLookup.get(filePath);
  if (!file || !file.hunks || file.hunks.length === 0) {
    return "";
  }

  const matchedHunk = findHunkForLine(file.hunks, lineNumber);
  if (!matchedHunk) {
    return renderDiffPreview(file.hunks);
  }

  return renderDiffPreview([matchedHunk], lineNumber);
}

function findHunkForLine(hunks, lineNumber) {
  if (lineNumber == null) {
    return hunks[0] || null;
  }

  return hunks.find((hunk) =>
    (hunk.lines || []).some((line) =>
      line.new_line_number === lineNumber || line.old_line_number === lineNumber
    )
  ) || null;
}

function statusKey(s) {
  const map = { Passed: "pass", Failed: "fail", Skipped: "skip" };
  return map[s] || s;
}

function loc(filePath, lineNumber) {
  if (!filePath) return "";
  const line = lineNumber ? `:${lineNumber}` : "";
  return `<span class="location">${esc(filePath)}${line}</span>`;
}

function esc(str) {
  if (typeof str !== "string") return String(str ?? "");
  const d = document.createElement("div");
  d.textContent = str;
  return d.innerHTML;
}

function openSettings() {
  activeView = "settings";
  render();
}

function openWorkspace() {
  activeView = "workspace";
  render();
}

function setSettingsCategory(category) {
  activeSettingsCategory = category;
  render();
}

function renderSettings() {
  document.getElementById("setting-theme-select").value = state.theme;
  document.getElementById("setting-time-format").value = uiSettings.timeFormat;
  document.getElementById("setting-wrap-diff-lines").checked = !!uiSettings.wrapDiffLines;
  document.getElementById("setting-show-diff-line-numbers").checked = !!uiSettings.showDiffLineNumbers;

  document.querySelectorAll(".settings-category").forEach((section) => {
    section.style.display = section.dataset.settingsCategory === activeSettingsCategory ? "block" : "none";
  });
}

function applyUiSettings() {
  document.body.classList.toggle("wrap-diff-lines", !!uiSettings.wrapDiffLines);
  document.body.classList.toggle("hide-diff-line-numbers", !uiSettings.showDiffLineNumbers);
}

function loadUiSettings() {
  try {
    const raw = window.localStorage.getItem(SETTINGS_STORAGE_KEY);
    if (!raw) return { ...defaultUiSettings };
    const parsed = JSON.parse(raw);
    const merged = { ...defaultUiSettings, ...parsed };
    if (
      typeof parsed.use24HourTime === "boolean" &&
      (parsed.timeFormat == null || parsed.timeFormat === "")
    ) {
      merged.timeFormat = parsed.use24HourTime ? "24h" : "system";
    }
    if (!["system", "24h", "12h"].includes(merged.timeFormat)) {
      merged.timeFormat = defaultUiSettings.timeFormat;
    }
    return merged;
  } catch (e) {
    console.warn("loadUiSettings:", e);
    return { ...defaultUiSettings };
  }
}

function saveUiSettings() {
  try {
    window.localStorage.setItem(SETTINGS_STORAGE_KEY, JSON.stringify(uiSettings));
  } catch (e) {
    console.warn("saveUiSettings:", e);
  }
}

function updateUiSetting(key, value) {
  uiSettings = { ...uiSettings, [key]: value };
  saveUiSettings();
  render();
}

function resetUiSettings() {
  uiSettings = { ...defaultUiSettings };
  saveUiSettings();
  setTheme("system");
}

// ─── Actions ───────────────────────────────────────────────────

async function addProject() {
  try {
    activeView = "workspace";
    state = await invoke("add_project");
    render();
  } catch (e) {
    console.error("add_project:", e);
  }
}

async function removeProject(id) {
  try {
    state = await invoke("remove_project", { projectId: id });
    delete historyPageByProjectId[id];
    delete activeRunIndexByProjectId[id];
    render();
  } catch (e) {
    console.error("remove_project:", e);
  }
}

async function selectProject(id) {
  try {
    activeView = "workspace";
    state = await invoke("select_project", { projectId: id });
    render();
  } catch (e) {
    console.error("select_project:", e);
  }
}

async function setTheme(theme) {
  try {
    state = await invoke("set_theme", { theme });
    render();
  } catch (e) {
    console.error("set_theme:", e);
  }
}

async function setSource(source) {
  try {
    state = await invoke("set_input_source", { source });
    render();
  } catch (e) {
    console.error("set_input_source:", e);
  }
}

async function captureClipboard() {
  try {
    const text = await navigator.clipboard.readText();
    state = await invoke("set_clipboard_diff", { diffText: text });
    render();
  } catch (e) {
    console.error("clipboard:", e);
  }
}

async function pickPatchFile() {
  try {
    state = await invoke("pick_patch_file");
    render();
  } catch (e) {
    console.error("pick_patch_file:", e);
  }
}

async function runVerification() {
  try {
    const activeProject = getActiveProject();
    if (activeProject) {
      activeProject.run_state = { kind: "running" };
      render();
    }
    state = await invoke("run_verification");
    const refreshedProject = getActiveProject();
    if (refreshedProject) {
      historyPageByProjectId[refreshedProject.id] = 0;
      activeRunIndexByProjectId[refreshedProject.id] = 0;
    }
    render();
  } catch (e) {
    const activeProject = getActiveProject();
    if (activeProject) {
      activeProject.run_state = { kind: "failed", data: String(e) };
      render();
    }
  }
}

window.addEventListener("DOMContentLoaded", init);
