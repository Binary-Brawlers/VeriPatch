const { invoke } = window.__TAURI__.core;

let state = null;

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

  // Theme switcher
  document.querySelectorAll(".theme-btn").forEach((btn) => {
    btn.addEventListener("click", () => setTheme(btn.dataset.theme));
  });
}

// ─── Menu bridge (called from Rust) ────────────────────────────

window.addProjectFromMenu = addProject;
window.setThemeFromMenu = function (t) { setTheme(t); };

// ─── Render ────────────────────────────────────────────────────

function render() {
  if (!state) return;

  applyTheme(state.theme);
  renderSidebar();

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
  document.querySelectorAll(".theme-btn").forEach((btn) => {
    btn.classList.toggle("active", btn.dataset.theme === theme);
  });
}

function renderSidebar() {
  const list = document.getElementById("project-list");
  const empty = document.getElementById("no-projects");

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
  const ids = ["result-idle", "result-running", "result-failed", "result-finished"];
  ids.forEach((id) => (document.getElementById(id).style.display = "none"));

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

  renderRunHistory(project.run_history || []);
}

function renderRunHistory(runHistory) {
  const section = document.getElementById("section-history");
  if (!runHistory || runHistory.length === 0) {
    section.innerHTML = "";
    return;
  }

  section.innerHTML = `
    <details class="collapsible-section" open>
      <summary class="section-header">
        <svg class="chevron-icon" viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" stroke-width="2"><polyline points="6 9 12 15 18 9"/></svg>
        Run History<span class="count-badge">${runHistory.length}</span>
      </summary>
      <div class="section-content">
        ${runHistory.map((entry, index) => {
          const verdict = entry.snapshot?.result?.verdict || "UNKNOWN";
          const verdictClass = `verdict-${String(verdict).toLowerCase()}`;
          return `<div class="history-item" data-run-index="${index}">
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
            <button class="btn btn-secondary btn-sm history-open-btn" data-run-index="${index}">Open</button>
          </div>`;
        }).join("")}
      </div>
    </details>
  `;

  section.querySelectorAll(".history-open-btn").forEach((btn) => {
    btn.addEventListener("click", (e) => {
      e.preventDefault();
      const idx = Number(btn.dataset.runIndex);
      openRunFromHistory(idx);
    });
  });
}

function openRunFromHistory(index) {
  const active = getActiveProject();
  if (!active || !active.run_history || !active.run_history[index]) return;
  const entry = active.run_history[index];
  active.run_state = { kind: "finished", data: entry.snapshot };
  render();
}

function formatRunTimestamp(value) {
  if (!value) return "Unknown time";
  const d = new Date(value);
  if (Number.isNaN(d.getTime())) return value;
  return d.toLocaleString(undefined, {
    year: "numeric",
    month: "short",
    day: "numeric",
    hour: "2-digit",
    minute: "2-digit",
  });
}

function renderSnapshot(snapshot) {
  const r = snapshot.result;
  const verdictClass = `verdict-${r.verdict.toLowerCase()}`;

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
    let diffHtml = "";
    if (hasHunks) {
      diffHtml = `<div class="diff-preview">${f.hunks.map((h) =>
        `<div class="diff-hunk"><div class="diff-hunk-header">${esc(h.header)}</div>${h.lines.map((l) => {
          const cls = l.kind === "Addition" ? "diff-add" : l.kind === "Deletion" ? "diff-del" : "diff-ctx";
          const prefix = l.kind === "Addition" ? "+" : l.kind === "Deletion" ? "-" : " ";
          const ln = l.kind === "Deletion"
            ? (l.old_line_number != null ? String(l.old_line_number).padStart(4) : "    ")
            : (l.new_line_number != null ? String(l.new_line_number).padStart(4) : "    ");
          return `<div class="diff-line ${cls}"><span class="diff-ln">${ln}</span><span class="diff-prefix">${prefix}</span><span class="diff-text">${esc(l.content)}</span></div>`;
        }).join("")}</div>`
      ).join("")}</div>`;
    }
    return `<details class="file-entry" id="file-entry-${i}">
      <summary class="file-item">
        <svg class="file-chevron" viewBox="0 0 24 24" width="12" height="12" fill="none" stroke="currentColor" stroke-width="2"><polyline points="6 9 12 15 18 9"/></svg>
        <span class="change-type ${ct}">${esc(f.change_type || "Mod")}</span>
        <span class="file-path"><span class="file-dir">${esc(dirPath)}</span><span class="file-name">${esc(fileName)}</span></span>
        <span class="diff-stat">+${f.additions} / -${f.deletions}</span>
      </summary>
      ${hasHunks ? diffHtml : '<div class="diff-preview"><div class="diff-empty">No diff content available</div></div>'}
    </details>`;
  });

  // Risky patterns
  renderSection("section-risky", "Risky Patterns", r.risky_patterns, (f) => {
    const sev = (f.severity || "low").toLowerCase();
    return `<div class="finding-item">
      <span class="severity severity-${sev}">${esc(f.severity)}</span>
      ${esc(f.message)}
      ${loc(f.file_path, f.line_number)}
    </div>`;
  });

  // Assumptions
  renderSection("section-assumptions", "Assumptions", r.assumptions, (a) =>
    `<div class="finding-item">${esc(a.message)}${loc(a.file_path, a.line_number)}</div>`
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

function getActiveProject() {
  if (!state || !state.active_project_id) return null;
  return state.projects.find((p) => p.id === state.active_project_id) || null;
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

// ─── Actions ───────────────────────────────────────────────────

async function addProject() {
  try {
    state = await invoke("add_project");
    render();
  } catch (e) {
    console.error("add_project:", e);
  }
}

async function removeProject(id) {
  try {
    state = await invoke("remove_project", { projectId: id });
    render();
  } catch (e) {
    console.error("remove_project:", e);
  }
}

async function selectProject(id) {
  try {
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
      renderResults({ ...activeProject, run_state: { kind: "running" } });
    }
    state = await invoke("run_verification");
    render();
  } catch (e) {
    const activeProject = getActiveProject();
    if (activeProject) {
      renderResults({ ...activeProject, run_state: { kind: "failed", data: String(e) } });
    }
  }
}

window.addEventListener("DOMContentLoaded", init);
