import { SETTINGS_NAV } from "../constants/ui";
import { formatLanguageLabel, languageClassName } from "../utils/app";
import { BackIcon, CloseIcon, LogoIcon, PlusIcon, SettingsIcon } from "./icons";

export function Sidebar({
  activeSettingsCategory,
  activeView,
  frontendState,
  onAddProject,
  onOpenSettings,
  onOpenWorkspace,
  onRemoveProject,
  onSelectProject,
  onSelectSettingsCategory,
}) {
  const projects = frontendState?.projects || [];
  const supportedLanguages = frontendState?.supported_languages || [];

  return (
    <aside id="sidebar">
      <div className="sidebar-header">
        <LogoIcon className="logo-icon" />
        <span className="logo-text">VeriPatch</span>
      </div>

      {activeView === "settings" ? (
        <div className="sidebar-section" id="sidebar-settings-section">
          <div className="sidebar-section-header">
            <span>Settings</span>
          </div>
          <nav className="settings-nav">
            {SETTINGS_NAV.map((item) => (
              <button
                key={item.id}
                className={`settings-nav-item ${
                  item.id === activeSettingsCategory ? "active" : ""
                }`}
                onClick={() => onSelectSettingsCategory(item.id)}
                type="button"
              >
                <span className="settings-nav-title">{item.title}</span>
                <span className="settings-nav-sub">{item.description}</span>
              </button>
            ))}
          </nav>
        </div>
      ) : (
        <>
          <div className="sidebar-section" id="sidebar-projects-section">
            <div className="sidebar-section-header">
              <span>Projects</span>
              <button
                className="icon-btn"
                id="btn-add-project"
                onClick={onAddProject}
                title="Add project"
                type="button"
              >
                <PlusIcon />
              </button>
            </div>

            {projects.length === 0 ? (
              <div className="empty-state" id="no-projects">
                <p>No projects yet</p>
                <button
                  className="btn btn-secondary btn-sm"
                  id="btn-add-first"
                  onClick={onAddProject}
                  type="button"
                >
                  Add a project
                </button>
              </div>
            ) : (
              <ul className="project-list" id="project-list">
                {projects.map((project) => {
                  const active = project.id === frontendState?.active_project_id;
                  return (
                    <li
                      key={project.id}
                      className={`project-item ${active ? "active" : ""}`}
                      onClick={() => onSelectProject(project.id)}
                    >
                      <div className="project-copy">
                        <span className="project-label">{project.name}</span>
                        <span
                          className={`language-badge project-language-badge ${languageClassName(
                            project.language,
                          )}`}
                        >
                          {formatLanguageLabel(project.language)}
                        </span>
                      </div>
                      <button
                        className="remove-btn"
                        onClick={(event) => {
                          event.stopPropagation();
                          onRemoveProject(project.id);
                        }}
                        title="Remove project"
                        type="button"
                      >
                        <CloseIcon />
                      </button>
                    </li>
                  );
                })}
              </ul>
            )}
          </div>

          <div className="sidebar-section sidebar-section-compact" id="sidebar-languages-section">
            <div className="sidebar-section-header">
              <span>Supported Languages</span>
            </div>

            <div className="supported-language-list" id="supported-language-list">
              {supportedLanguages.length === 0 ? (
                <div className="sidebar-empty-copy">No supported languages registered.</div>
              ) : (
                supportedLanguages.map((language) => (
                  <div className="supported-language-item" key={language.id}>
                    <div className="supported-language-head">
                      <span
                        className={`language-badge supported-language-badge ${languageClassName(
                          language.id,
                        )}`}
                      >
                        {language.name}
                      </span>
                    </div>
                    <div className="supported-language-meta">
                      {(language.manifests || []).join(" • ")}
                    </div>
                  </div>
                ))
              )}
            </div>
          </div>
        </>
      )}

      <div className="sidebar-footer">
        {activeView === "settings" ? (
          <button
            className="btn btn-secondary btn-sm"
            id="btn-close-settings"
            onClick={onOpenWorkspace}
            type="button"
          >
            <BackIcon />
            Back to Workspace
          </button>
        ) : (
          <button
            className="btn btn-secondary btn-sm"
            id="btn-open-settings"
            onClick={onOpenSettings}
            type="button"
          >
            <SettingsIcon />
            Settings
          </button>
        )}
      </div>
    </aside>
  );
}
