import { HISTORY_PAGE_SIZE, SOURCE_OPTIONS } from "../constants/ui";
import {
  buildDiffLookup,
  formatLanguageLabel,
  formatPullRequestLabel,
  formatRunTimestamp,
  getFindingPreview,
  languageClassName,
  statusKey,
} from "../utils/app";
import { ChevronIcon, ErrorIcon, ExportIcon, PlayIcon } from "./icons";

export function WorkspaceView({
  activeProject,
  activeRunIndex,
  displayedRunState,
  exportStatus,
  historyPageByProjectId,
  onCaptureClipboard,
  onClosePullRequest,
  onExportMarkdownReport,
  onHistoryPageChange,
  onMergePullRequest,
  onOpenRunFromHistory,
  onPickPatchFile,
  onRefreshPullRequests,
  onRunVerification,
  onSelectPullRequest,
  onSetSource,
  uiSettings,
}) {
  const canExport = displayedRunState?.kind === "finished";

  return (
    <div id="view-project">
      <div className="toolbar">
        <div className="toolbar-left">
          <div className="toolbar-title-row">
            <h2 className="toolbar-title" id="project-name">
              {activeProject.name}
            </h2>
            <span
              className={`language-badge toolbar-language-badge ${languageClassName(
                activeProject.language,
              )}`}
              id="project-language-badge"
            >
              {formatLanguageLabel(activeProject.language)}
            </span>
          </div>
          <span className="toolbar-path" id="project-path">
            {activeProject.repo_path}
          </span>
        </div>

        <div className="toolbar-right">
          <button
            className="btn btn-secondary"
            disabled={!canExport}
            id="btn-export-markdown"
            onClick={onExportMarkdownReport}
            type="button"
          >
            <ExportIcon />
            Export Markdown
          </button>
          <button className="btn btn-primary" id="btn-run" onClick={onRunVerification} type="button">
            <PlayIcon />
            Run Verification
          </button>
          <span className="toolbar-status" id="export-status">
            {exportStatus}
          </span>
        </div>
      </div>

      <div className="config-bar">
        <div className="config-group">
          <label className="config-label">Source</label>
          <div className="segmented-control">
            {SOURCE_OPTIONS.map((option) => (
              <button
                key={option.id}
                className={`seg-btn ${
                  activeProject.input_source === option.id ? "active" : ""
                }`}
                data-source={option.id}
                onClick={() => onSetSource(option.id)}
                type="button"
              >
                {option.label}
              </button>
            ))}
          </div>
        </div>

        {activeProject.input_source === "clipboard_diff" ? (
          <div className="config-group" id="clipboard-actions">
            <button
              className="btn btn-secondary btn-sm"
              id="btn-clipboard"
              onClick={onCaptureClipboard}
              type="button"
            >
              Paste from clipboard
            </button>
            <span className="config-hint" id="clipboard-hint">
              {activeProject.clipboard_diff
                ? `${activeProject.clipboard_diff.split("\n").length} lines loaded`
                : "No diff loaded"}
            </span>
          </div>
        ) : null}

        {activeProject.input_source === "patch_file" ? (
          <div className="config-group" id="patch-actions">
            <button
              className="btn btn-secondary btn-sm"
              id="btn-patch"
              onClick={onPickPatchFile}
              type="button"
            >
              Choose file…
            </button>
            <span className="config-hint" id="patch-hint">
              {activeProject.patch_path || "No file selected"}
            </span>
          </div>
        ) : null}

        {activeProject.input_source === "pull_request" ? (
          <div className="config-group config-group-stack" id="pull-request-actions">
            <div className="config-group">
              <button
                className="btn btn-secondary btn-sm"
                disabled={!!activeProject.pull_request_busy}
                id="btn-pr-refresh"
                onClick={onRefreshPullRequests}
                type="button"
              >
                Refresh PRs
              </button>

              <select
                aria-label="Pull request selector"
                className="config-select"
                disabled={
                  !!activeProject.pull_request_busy || (activeProject.pull_requests || []).length === 0
                }
                id="pull-request-select"
                onChange={(event) => {
                  const value = event.target.value;
                  onSelectPullRequest(value ? Number(value) : null);
                }}
                value={activeProject.selected_pull_request_number ?? ""}
              >
                {(activeProject.pull_requests || []).length > 0 ? (
                  activeProject.pull_requests.map((pullRequest) => (
                    <option key={pullRequest.number} value={pullRequest.number}>
                      {formatPullRequestLabel(pullRequest)}
                    </option>
                  ))
                ) : (
                  <option value="">No open pull requests</option>
                )}
              </select>

              <button
                className="btn btn-secondary btn-sm"
                disabled={
                  !!activeProject.pull_request_busy || activeProject.selected_pull_request_number == null
                }
                id="btn-pr-merge"
                onClick={onMergePullRequest}
                type="button"
              >
                Merge
              </button>
              <button
                className="btn btn-secondary btn-sm"
                disabled={
                  !!activeProject.pull_request_busy || activeProject.selected_pull_request_number == null
                }
                id="btn-pr-close"
                onClick={onClosePullRequest}
                type="button"
              >
                Close
              </button>
            </div>
            <span className="config-hint" id="pull-request-hint">
              {activeProject.pull_request_message ||
                ((activeProject.pull_requests || []).length > 0
                  ? "Select a pull request diff to verify."
                  : "Load open pull requests for this repository.")}
            </span>
            {activeProject.pull_request_error ? (
              <span className="config-error" id="pull-request-error">
                {activeProject.pull_request_error}
              </span>
            ) : null}
          </div>
        ) : null}
      </div>

      <ResultsView
        activeProject={activeProject}
        activeRunIndex={activeRunIndex}
        displayedRunState={displayedRunState}
        historyPageByProjectId={historyPageByProjectId}
        onHistoryPageChange={onHistoryPageChange}
        onOpenRunFromHistory={onOpenRunFromHistory}
        uiSettings={uiSettings}
      />
    </div>
  );
}

function ResultsView({
  activeProject,
  activeRunIndex,
  displayedRunState,
  historyPageByProjectId,
  onHistoryPageChange,
  onOpenRunFromHistory,
  uiSettings,
}) {
  const runHistory = activeProject.run_history || [];
  const hasHistory = runHistory.length > 0;

  return (
    <div className={hasHistory ? "has-history" : ""} id="results-area">
      <aside id="results-history-panel">
        {hasHistory ? (
          <RunHistorySection
            activeProject={activeProject}
            activeRunIndex={activeRunIndex}
            historyPage={historyPageByProjectId[activeProject.id] || 0}
            onHistoryPageChange={onHistoryPageChange}
            onOpenRunFromHistory={onOpenRunFromHistory}
            uiSettings={uiSettings}
          />
        ) : null}
      </aside>

      <section id="results-content-panel">
        {displayedRunState?.kind === "running" ? (
          <div className="result-placeholder" id="result-running">
            <div className="spinner"></div>
            <p>Running checks — compile, lint, tests, security rules…</p>
          </div>
        ) : null}

        {displayedRunState?.kind === "failed" ? (
          <div className="result-placeholder" id="result-failed">
            <ErrorIcon />
            <p className="text-danger" id="error-message">
              {displayedRunState.data}
            </p>
          </div>
        ) : null}

        {displayedRunState?.kind === "finished" ? (
          <FinishedResults snapshot={displayedRunState.data} />
        ) : null}

        {!displayedRunState || displayedRunState.kind === "idle" ? (
          <div className="result-placeholder" id="result-idle">
            <PlayIcon width={32} height={32} opacity={0.25} strokeWidth={1.5} />
            <p>Configure your diff source and run the verification pipeline.</p>
          </div>
        ) : null}
      </section>
    </div>
  );
}

function RunHistorySection({
  activeProject,
  activeRunIndex,
  historyPage,
  onHistoryPageChange,
  onOpenRunFromHistory,
  uiSettings,
}) {
  const runHistory = activeProject.run_history || [];
  const totalPages = Math.max(1, Math.ceil(runHistory.length / HISTORY_PAGE_SIZE));
  const currentPage = Math.min(historyPage, totalPages - 1);
  const start = currentPage * HISTORY_PAGE_SIZE;
  const end = Math.min(start + HISTORY_PAGE_SIZE, runHistory.length);
  const pageItems = runHistory.slice(start, end);

  return (
    <div className="result-section" id="section-history">
      <details className="collapsible-section" open>
        <summary className="section-header">
          <ChevronIcon className="chevron-icon" />
          Run History
          <span className="count-badge">{runHistory.length}</span>
        </summary>
        <div className="section-content">
          {pageItems.map((entry, pageIndex) => {
            const index = start + pageIndex;
            const verdict = entry.snapshot?.result?.verdict || "UNKNOWN";
            const isActive = index === activeRunIndex;
            return (
              <button
                aria-pressed={isActive}
                className={`history-item ${isActive ? "active" : ""}`}
                data-run-index={index}
                key={entry.run_id || `${entry.ran_at}-${index}`}
                onClick={() => onOpenRunFromHistory(index)}
                type="button"
              >
                <div className="history-main">
                  <div className="history-head">
                    <span className="history-run-id">{entry.run_id || `run-${index + 1}`}</span>
                    <span className="history-time">
                      {formatRunTimestamp(entry.ran_at, uiSettings)}
                    </span>
                  </div>
                  <div className="history-meta">
                    <span className="history-source">
                      {entry.snapshot?.source_label || "Unknown source"}
                    </span>
                    <span
                      className={`history-verdict verdict-${String(verdict).toLowerCase()}`}
                    >
                      {verdict}
                    </span>
                    <span className="history-score">
                      {entry.snapshot?.result?.score ?? "-"}/100
                    </span>
                  </div>
                </div>
                <span className="history-item-action">{isActive ? "Viewing" : "Open"}</span>
              </button>
            );
          })}

          {totalPages > 1 ? (
            <div className="history-pagination">
              <div className="history-page-status">
                Showing {start + 1}-{end} of {runHistory.length}
              </div>
              <div className="history-page-actions">
                <button
                  className="btn btn-secondary btn-sm history-page-btn"
                  disabled={currentPage === 0}
                  onClick={() => onHistoryPageChange(activeProject.id, currentPage - 1)}
                  type="button"
                >
                  Previous
                </button>
                <span className="history-page-indicator">
                  Page {currentPage + 1} / {totalPages}
                </span>
                <button
                  className="btn btn-secondary btn-sm history-page-btn"
                  disabled={currentPage >= totalPages - 1}
                  onClick={() => onHistoryPageChange(activeProject.id, currentPage + 1)}
                  type="button"
                >
                  Next
                </button>
              </div>
            </div>
          ) : null}
        </div>
      </details>
    </div>
  );
}

function FinishedResults({ snapshot }) {
  const result = snapshot.result;
  const diffLookup = buildDiffLookup(result.diff.files || []);

  return (
    <div id="result-finished">
      <div className="metrics-grid" id="metrics">
        <div className="metric-card">
          <span className="label">Verdict</span>
          <span className={`val verdict-${String(result.verdict).toLowerCase()}`}>
            {result.verdict}
          </span>
        </div>
        <div className="metric-card">
          <span className="label">Score</span>
          <span className="val">
            {result.score}
            <span style={{ fontSize: 12, fontWeight: 400, color: "var(--text-tertiary)" }}>
              {" "}
              / 100
            </span>
          </span>
        </div>
        <div className="metric-card">
          <span className="label">Source</span>
          <span className="val" style={{ fontSize: 14 }}>
            {snapshot.source_label}
          </span>
        </div>
        <div className="metric-card">
          <span className="label">Scope</span>
          <span className="val" style={{ fontSize: 14 }}>
            {result.diff.files.length} files{" "}
            <span style={{ fontSize: 12, fontWeight: 400, color: "var(--text-tertiary)" }}>
              +{result.diff.total_additions} / -{result.diff.total_deletions}
            </span>
          </span>
        </div>
      </div>

      <Section title="Checks" items={result.checks}>
        {result.checks.map((check, index) => {
          const status = statusKey(check.status);
          return (
            <div className="check-item" key={`${check.name}-${index}`}>
              <div className={`check-status ${status}`}>
                {status === "pass" ? "✓" : status === "fail" ? "✗" : "—"}
              </div>
              <div className="check-info">
                <div className="check-name">{check.name}</div>
                <div className="check-summary">{check.summary}</div>
              </div>
            </div>
          );
        })}
      </Section>

      <Section title="Changed Files" items={result.diff.files}>
        {result.diff.files.map((file, index) => {
          const changeType = String(file.change_type || "Modified").toLowerCase();
          const filePath = file.new_path || file.old_path || "<unknown>";
          const fileName = filePath.split("/").pop();
          const slashIndex = filePath.lastIndexOf("/");
          const dirPath = slashIndex >= 0 ? filePath.slice(0, slashIndex + 1) : "";
          return (
            <details className="file-entry" id={`file-entry-${index}`} key={`${filePath}-${index}`}>
              <summary className="file-item">
                <ChevronIcon className="file-chevron" width={12} height={12} />
                <span className={`change-type ${changeType}`}>{file.change_type || "Mod"}</span>
                <span className="file-path">
                  <span className="file-dir">{dirPath}</span>
                  <span className="file-name">{fileName}</span>
                </span>
                <span className="diff-stat">
                  +{file.additions} / -{file.deletions}
                </span>
              </summary>
              {file.hunks?.length ? (
                <DiffPreview hunks={file.hunks} />
              ) : (
                <EmptyDiffPreview />
              )}
            </details>
          );
        })}
      </Section>

      <Section
        emptyMessage="No risky patterns detected in added lines."
        items={result.risky_patterns}
        title="Risky Patterns"
      >
        {result.risky_patterns.map((finding, index) => (
          <FindingItem
            badge={
              <span
                className={`severity severity-${String(finding.severity || "low").toLowerCase()}`}
              >
                {finding.severity}
              </span>
            }
            diffLookup={diffLookup}
            filePath={finding.file_path}
            key={`${finding.file_path}-${finding.line_number}-${index}`}
            lineNumber={finding.line_number}
            message={finding.message}
          />
        ))}
      </Section>

      <Section
        emptyMessage="No assumptions detected in added lines."
        items={result.assumptions}
        title="Assumptions"
      >
        {result.assumptions.map((assumption, index) => (
          <FindingItem
            diffLookup={diffLookup}
            filePath={assumption.file_path}
            key={`${assumption.file_path}-${assumption.line_number}-${index}`}
            lineNumber={assumption.line_number}
            message={assumption.message}
          />
        ))}
      </Section>

      <Section items={result.dependency_notes} title="Dependencies">
        {result.dependency_notes.map((note, index) => (
          <div className="finding-item" key={`${note}-${index}`}>
            {note}
          </div>
        ))}
      </Section>

      <Section items={result.warnings} title="Warnings">
        {result.warnings.map((warning, index) => (
          <div className="finding-item" key={`${warning}-${index}`}>
            {warning}
          </div>
        ))}
      </Section>
    </div>
  );
}

function Section({ children, emptyMessage = null, items, title }) {
  if (!items?.length && !emptyMessage) {
    return null;
  }

  return (
    <div className="result-section">
      <details className="collapsible-section" open>
        <summary className="section-header">
          <ChevronIcon className="chevron-icon" />
          {title}
          <span className="count-badge">{items?.length || 0}</span>
        </summary>
        <div className="section-content">
          {items?.length ? children : <div className="section-empty-state">{emptyMessage}</div>}
        </div>
      </details>
    </div>
  );
}

function FindingItem({ badge = null, diffLookup, filePath, lineNumber, message }) {
  const snippet = getFindingPreview(diffLookup, filePath, lineNumber);

  return (
    <details className="finding-item">
      <summary className="finding-summary">
        <ChevronIcon className="finding-chevron" width={12} height={12} />
        <div className="finding-copy">
          {badge}
          {message}
          {filePath ? (
            <span className="location">
              {filePath}
              {lineNumber ? `:${lineNumber}` : ""}
            </span>
          ) : null}
        </div>
      </summary>
      {snippet ? (
        <DiffPreview highlightedLineNumber={snippet.highlightedLineNumber} hunks={snippet.hunks} />
      ) : (
        <EmptyDiffPreview />
      )}
    </details>
  );
}

function EmptyDiffPreview() {
  return (
    <div className="diff-preview">
      <div className="diff-empty">No diff content available</div>
    </div>
  );
}

function DiffPreview({ highlightedLineNumber = null, hunks }) {
  return (
    <div className="diff-preview">
      {(hunks || []).map((hunk, index) => (
        <div className="diff-hunk" key={`${hunk.header}-${index}`}>
          <div className="diff-hunk-header">{hunk.header}</div>
          {(hunk.lines || []).map((line, lineIndex) => {
            const kind =
              line.kind === "Addition"
                ? "diff-add"
                : line.kind === "Deletion"
                  ? "diff-del"
                  : "diff-ctx";
            const prefix =
              line.kind === "Addition" ? "+" : line.kind === "Deletion" ? "-" : " ";
            const visibleLineNumber =
              line.kind === "Deletion" ? line.old_line_number : line.new_line_number;
            const lineNumberText =
              visibleLineNumber != null ? String(visibleLineNumber).padStart(4) : "    ";
            const focus = highlightedLineNumber != null && visibleLineNumber === highlightedLineNumber;

            return (
              <div className={`diff-line ${kind}${focus ? " diff-focus" : ""}`} key={lineIndex}>
                <span className="diff-ln">{lineNumberText}</span>
                <span className="diff-prefix">{prefix}</span>
                <span className="diff-text">{line.content}</span>
              </div>
            );
          })}
        </div>
      ))}
    </div>
  );
}
