import { DEFAULT_UI_SETTINGS, SETTINGS_STORAGE_KEY } from "../constants/ui";

export function getActiveProject(frontendState) {
  if (!frontendState?.active_project_id) {
    return null;
  }

  return (
    frontendState.projects.find((project) => project.id === frontendState.active_project_id) || null
  );
}

export function getCurrentRunIndex(project, activeRunIndexByProjectId) {
  if (!project || project.run_state?.kind !== "finished" || !project.run_history?.length) {
    return null;
  }

  const storedIndex = activeRunIndexByProjectId[project.id];
  if (
    Number.isInteger(storedIndex) &&
    storedIndex >= 0 &&
    storedIndex < project.run_history.length
  ) {
    return storedIndex;
  }

  return 0;
}

export function getDisplayedRunState(project, activeRunIndex) {
  if (!project) {
    return null;
  }

  if (project.run_state?.kind !== "finished") {
    return project.run_state;
  }

  const historyEntry =
    activeRunIndex != null ? project.run_history?.[activeRunIndex] : project.run_history?.[0];
  return historyEntry ? { kind: "finished", data: historyEntry.snapshot } : project.run_state;
}

export function buildDiffLookup(files) {
  const lookup = new Map();
  for (const file of files || []) {
    const filePath = file.new_path || file.old_path;
    if (filePath) {
      lookup.set(filePath, file);
    }
  }
  return lookup;
}

export function getFindingPreview(diffLookup, filePath, lineNumber) {
  if (!filePath) {
    return null;
  }

  const file = diffLookup.get(filePath);
  if (!file?.hunks?.length) {
    return null;
  }

  const matchedHunk = findHunkForLine(file.hunks, lineNumber);
  return {
    hunks: matchedHunk ? [matchedHunk] : file.hunks,
    highlightedLineNumber: lineNumber ?? null,
  };
}

export function findHunkForLine(hunks, lineNumber) {
  if (lineNumber == null) {
    return hunks[0] || null;
  }

  return (
    hunks.find((hunk) =>
      (hunk.lines || []).some(
        (line) => line.new_line_number === lineNumber || line.old_line_number === lineNumber,
      ),
    ) || null
  );
}

export function formatRunTimestamp(value, uiSettings) {
  if (!value) {
    return "Unknown time";
  }

  const date = new Date(value);
  if (Number.isNaN(date.getTime())) {
    return value;
  }

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

  return date.toLocaleString(undefined, options);
}

export function formatPullRequestLabel(pullRequest) {
  const draftPrefix = pullRequest.is_draft ? "Draft · " : "";
  return `#${pullRequest.number} · ${draftPrefix}${pullRequest.title} · ${pullRequest.head_ref_name} → ${pullRequest.base_ref_name} · ${pullRequest.author}`;
}

export function formatLanguageLabel(language) {
  const labels = {
    rust: "Rust",
    typescript: "TypeScript",
    unsupported: "Unsupported",
  };
  return labels[language] || String(language || "Unknown");
}

export function languageClassName(language) {
  return `language-${String(language || "unsupported").toLowerCase()}`;
}

export function statusKey(status) {
  const map = { Passed: "pass", Failed: "fail", Skipped: "skip" };
  return map[status] || String(status || "").toLowerCase();
}

export function loadUiSettings() {
  try {
    const raw = window.localStorage.getItem(SETTINGS_STORAGE_KEY);
    if (!raw) {
      return { ...DEFAULT_UI_SETTINGS };
    }

    const parsed = JSON.parse(raw);
    const merged = { ...DEFAULT_UI_SETTINGS, ...parsed };
    if (
      typeof parsed.use24HourTime === "boolean" &&
      (parsed.timeFormat == null || parsed.timeFormat === "")
    ) {
      merged.timeFormat = parsed.use24HourTime ? "24h" : "system";
    }
    if (!["system", "24h", "12h"].includes(merged.timeFormat)) {
      merged.timeFormat = DEFAULT_UI_SETTINGS.timeFormat;
    }
    return merged;
  } catch (error) {
    console.warn("loadUiSettings:", error);
    return { ...DEFAULT_UI_SETTINGS };
  }
}

export function saveUiSettings(uiSettings) {
  try {
    window.localStorage.setItem(SETTINGS_STORAGE_KEY, JSON.stringify(uiSettings));
  } catch (error) {
    console.warn("saveUiSettings:", error);
  }
}
