import { SHORTCUTS } from "../constants/ui";

export function SettingsView({
  activeSettingsCategory,
  onReset,
  onSetTheme,
  onUpdateUiSetting,
  theme,
  uiSettings,
}) {
  return (
    <div className="settings-view" id="view-settings">
      <div className="settings-topbar">
        <div className="settings-topbar-copy">
          <h2 className="toolbar-title">Settings</h2>
          <span className="toolbar-path">Customize VeriPatch behavior</span>
        </div>
        <button
          className="btn btn-secondary btn-sm"
          id="btn-settings-reset"
          onClick={onReset}
          type="button"
        >
          Restore defaults
        </button>
      </div>

      <div className="settings-page">
        <section className="settings-panel">
          {activeSettingsCategory === "general" ? (
            <div className="settings-category" data-settings-category="general">
              <h3 className="settings-section-title">General</h3>
              <p className="settings-desc">Control app-wide look and time formatting.</p>

              <div className="settings-row">
                <div>
                  <div className="settings-row-title">Theme</div>
                  <div className="settings-row-desc">Choose how VeriPatch looks across the app.</div>
                </div>
                <div className="settings-control">
                  <select
                    className="settings-select"
                    id="setting-theme-select"
                    onChange={(event) => onSetTheme(event.target.value)}
                    value={theme}
                  >
                    <option value="system">System</option>
                    <option value="light">Light</option>
                    <option value="dark">Dark</option>
                  </select>
                </div>
              </div>

              <div className="settings-row">
                <div>
                  <div className="settings-row-title">Time format</div>
                  <div className="settings-row-desc">
                    Choose how run timestamps are displayed.
                  </div>
                </div>
                <div className="settings-control">
                  <select
                    className="settings-select"
                    id="setting-time-format"
                    onChange={(event) => onUpdateUiSetting("timeFormat", event.target.value)}
                    value={uiSettings.timeFormat}
                  >
                    <option value="system">System default</option>
                    <option value="24h">24-hour</option>
                    <option value="12h">12-hour</option>
                  </select>
                </div>
              </div>
            </div>
          ) : null}

          {activeSettingsCategory === "diff" ? (
            <div className="settings-category" data-settings-category="diff">
              <h3 className="settings-section-title">Diff View</h3>
              <p className="settings-desc">Set default behavior for diff previews.</p>

              <label className="settings-row settings-row-toggle">
                <div>
                  <div className="settings-row-title">Diff line wrapping</div>
                  <div className="settings-row-desc">
                    Wrap long lines in inline diff previews.
                  </div>
                </div>
                <div className="settings-control">
                  <input
                    checked={!!uiSettings.wrapDiffLines}
                    id="setting-wrap-diff-lines"
                    onChange={(event) =>
                      onUpdateUiSetting("wrapDiffLines", event.target.checked)
                    }
                    type="checkbox"
                  />
                </div>
              </label>

              <label className="settings-row settings-row-toggle">
                <div>
                  <div className="settings-row-title">Line numbers</div>
                  <div className="settings-row-desc">
                    Show line numbers and +/- markers in diff preview.
                  </div>
                </div>
                <div className="settings-control">
                  <input
                    checked={!!uiSettings.showDiffLineNumbers}
                    id="setting-show-diff-line-numbers"
                    onChange={(event) =>
                      onUpdateUiSetting("showDiffLineNumbers", event.target.checked)
                    }
                    type="checkbox"
                  />
                </div>
              </label>
            </div>
          ) : null}

          {activeSettingsCategory === "shortcuts" ? (
            <div className="settings-category" data-settings-category="shortcuts">
              <h3 className="settings-section-title">Keyboard Shortcuts</h3>
              <p className="settings-desc">Shortcuts come from the native menu bar.</p>

              <div className="shortcut-list">
                {SHORTCUTS.map(([label, shortcut]) => (
                  <div className="shortcut-item" key={label}>
                    <span>{label}</span>
                    <kbd>{shortcut}</kbd>
                  </div>
                ))}
              </div>
            </div>
          ) : null}
        </section>
      </div>
    </div>
  );
}
