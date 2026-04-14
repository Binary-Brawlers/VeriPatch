export const HISTORY_PAGE_SIZE = 10;
export const SETTINGS_STORAGE_KEY = "veripatch.ui.settings.v1";

export const DEFAULT_UI_SETTINGS = {
  timeFormat: "system",
  wrapDiffLines: false,
  showDiffLineNumbers: true,
};

export const SOURCE_OPTIONS = [
  { id: "current_working_tree", label: "Working Tree" },
  { id: "clipboard_diff", label: "Clipboard" },
  { id: "patch_file", label: "Patch File" },
  { id: "pull_request", label: "Pull Request" },
];

export const SETTINGS_NAV = [
  {
    id: "general",
    title: "General",
    description: "Theme and time format",
  },
  {
    id: "diff",
    title: "Diff View",
    description: "Preview behavior",
  },
  {
    id: "shortcuts",
    title: "Keyboard",
    description: "Native shortcuts",
  },
];

export const SHORTCUTS = [
  ["Add project", "Cmd/Ctrl + O"],
  ["Open settings", "Cmd/Ctrl + ,"],
  ["Undo", "Cmd/Ctrl + Z"],
  ["Redo", "Cmd/Ctrl + Shift + Z"],
  ["Copy selection", "Cmd/Ctrl + C"],
  ["Paste", "Cmd/Ctrl + V"],
];
