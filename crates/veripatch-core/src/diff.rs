//! Unified diff and patch file parsing.

use anyhow::{Result, bail};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParsedDiff {
    pub files: Vec<ChangedFile>,
    pub total_additions: usize,
    pub total_deletions: usize,
}

impl ParsedDiff {
    pub fn changed_paths(&self) -> Vec<String> {
        self.files.iter().map(|file| file.display_path()).collect()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChangedFile {
    pub old_path: Option<String>,
    pub new_path: Option<String>,
    pub change_type: FileChangeType,
    pub additions: usize,
    pub deletions: usize,
    pub hunks: Vec<DiffHunk>,
}

impl ChangedFile {
    pub fn display_path(&self) -> String {
        self.new_path
            .clone()
            .or_else(|| self.old_path.clone())
            .unwrap_or_else(|| "<unknown>".to_string())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum FileChangeType {
    Added,
    Modified,
    Deleted,
    Renamed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiffHunk {
    pub header: String,
    pub lines: Vec<DiffLine>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiffLine {
    pub kind: DiffLineKind,
    pub content: String,
    pub old_line_number: Option<usize>,
    pub new_line_number: Option<usize>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DiffLineKind {
    Context,
    Addition,
    Deletion,
}

pub fn parse_unified_diff(input: &str) -> Result<ParsedDiff> {
    if input.trim().is_empty() {
        bail!("diff input is empty");
    }

    let mut files = Vec::new();
    let mut current_file: Option<ChangedFile> = None;
    let mut current_hunk: Option<DiffHunk> = None;
    let mut old_line = 0usize;
    let mut new_line = 0usize;

    for raw_line in input.lines() {
        if let Some(paths) = raw_line.strip_prefix("diff --git ") {
            finalize_hunk(&mut current_file, &mut current_hunk);
            finalize_file(&mut files, &mut current_file);

            let mut parts = paths.split_whitespace();
            let old_path = parts.next().map(normalize_diff_path);
            let new_path = parts.next().map(normalize_diff_path);

            current_file = Some(ChangedFile {
                old_path,
                new_path,
                change_type: FileChangeType::Modified,
                additions: 0,
                deletions: 0,
                hunks: Vec::new(),
            });
            continue;
        }

        if let Some(path) = raw_line.strip_prefix("--- ") {
            let file = current_file.get_or_insert_with(empty_file);
            file.old_path = normalize_optional_path(path.trim());
            continue;
        }

        if let Some(path) = raw_line.strip_prefix("+++ ") {
            let file = current_file.get_or_insert_with(empty_file);
            file.new_path = normalize_optional_path(path.trim());
            file.change_type =
                classify_change_type(file.old_path.as_deref(), file.new_path.as_deref());
            continue;
        }

        if raw_line.starts_with("rename from ") {
            let file = current_file.get_or_insert_with(empty_file);
            file.old_path = Some(
                raw_line
                    .trim_start_matches("rename from ")
                    .trim()
                    .to_string(),
            );
            file.change_type = FileChangeType::Renamed;
            continue;
        }

        if raw_line.starts_with("rename to ") {
            let file = current_file.get_or_insert_with(empty_file);
            file.new_path = Some(raw_line.trim_start_matches("rename to ").trim().to_string());
            file.change_type = FileChangeType::Renamed;
            continue;
        }

        if raw_line.starts_with("@@") {
            finalize_hunk(&mut current_file, &mut current_hunk);

            let (parsed_old_line, parsed_new_line) = parse_hunk_header(raw_line)?;
            old_line = parsed_old_line;
            new_line = parsed_new_line;
            current_hunk = Some(DiffHunk {
                header: raw_line.to_string(),
                lines: Vec::new(),
            });
            continue;
        }

        if let Some(hunk) = current_hunk.as_mut() {
            match raw_line.as_bytes().first().copied() {
                Some(b'+') if !raw_line.starts_with("+++") => {
                    hunk.lines.push(DiffLine {
                        kind: DiffLineKind::Addition,
                        content: raw_line[1..].to_string(),
                        old_line_number: None,
                        new_line_number: Some(new_line),
                    });

                    if let Some(file) = current_file.as_mut() {
                        file.additions += 1;
                    }

                    new_line += 1;
                }
                Some(b'-') if !raw_line.starts_with("---") => {
                    hunk.lines.push(DiffLine {
                        kind: DiffLineKind::Deletion,
                        content: raw_line[1..].to_string(),
                        old_line_number: Some(old_line),
                        new_line_number: None,
                    });

                    if let Some(file) = current_file.as_mut() {
                        file.deletions += 1;
                    }

                    old_line += 1;
                }
                _ => {
                    let content = raw_line.strip_prefix(' ').unwrap_or(raw_line).to_string();
                    hunk.lines.push(DiffLine {
                        kind: DiffLineKind::Context,
                        content,
                        old_line_number: Some(old_line),
                        new_line_number: Some(new_line),
                    });

                    if !raw_line.starts_with("\\ No newline at end of file") {
                        old_line += 1;
                        new_line += 1;
                    }
                }
            }
        }
    }

    finalize_hunk(&mut current_file, &mut current_hunk);
    finalize_file(&mut files, &mut current_file);

    if files.is_empty() {
        bail!("no file changes were found in the diff");
    }

    let total_additions = files.iter().map(|file| file.additions).sum();
    let total_deletions = files.iter().map(|file| file.deletions).sum();

    Ok(ParsedDiff {
        files,
        total_additions,
        total_deletions,
    })
}

fn empty_file() -> ChangedFile {
    ChangedFile {
        old_path: None,
        new_path: None,
        change_type: FileChangeType::Modified,
        additions: 0,
        deletions: 0,
        hunks: Vec::new(),
    }
}

fn finalize_hunk(current_file: &mut Option<ChangedFile>, current_hunk: &mut Option<DiffHunk>) {
    if let (Some(file), Some(hunk)) = (current_file.as_mut(), current_hunk.take()) {
        file.hunks.push(hunk);
    }
}

fn finalize_file(files: &mut Vec<ChangedFile>, current_file: &mut Option<ChangedFile>) {
    if let Some(mut file) = current_file.take() {
        file.change_type = classify_change_type(file.old_path.as_deref(), file.new_path.as_deref());
        files.push(file);
    }
}

fn parse_hunk_header(header: &str) -> Result<(usize, usize)> {
    let mut parts = header.split_whitespace();
    let _start = parts.next();
    let old_range = parts
        .next()
        .ok_or_else(|| anyhow::anyhow!("missing old hunk range"))?;
    let new_range = parts
        .next()
        .ok_or_else(|| anyhow::anyhow!("missing new hunk range"))?;

    Ok((parse_range_start(old_range)?, parse_range_start(new_range)?))
}

fn parse_range_start(range: &str) -> Result<usize> {
    let normalized = range.trim_start_matches(['-', '+']);
    let start = normalized.split(',').next().unwrap_or("1");
    Ok(start.parse()?)
}

fn normalize_optional_path(path: &str) -> Option<String> {
    if path == "/dev/null" {
        return None;
    }

    Some(normalize_diff_path(path))
}

fn normalize_diff_path(path: &str) -> String {
    path.trim_start_matches("a/")
        .trim_start_matches("b/")
        .to_string()
}

fn classify_change_type(old_path: Option<&str>, new_path: Option<&str>) -> FileChangeType {
    match (old_path, new_path) {
        (None, Some(_)) => FileChangeType::Added,
        (Some(_), None) => FileChangeType::Deleted,
        (Some(old), Some(new)) if old != new => FileChangeType::Renamed,
        _ => FileChangeType::Modified,
    }
}

#[cfg(test)]
mod tests {
    use super::{DiffLineKind, FileChangeType, parse_unified_diff};

    #[test]
    fn parses_basic_unified_diff() {
        let diff = "diff --git a/src/lib.rs b/src/lib.rs\nindex 1111111..2222222 100644\n--- a/src/lib.rs\n+++ b/src/lib.rs\n@@ -1,2 +1,3 @@\n pub fn verify() {\n-    panic!(\"old\");\n+    println!(\"new\");\n+    println!(\"more\");\n }\n";

        let parsed = parse_unified_diff(diff).expect("diff should parse");
        assert_eq!(parsed.files.len(), 1);
        assert_eq!(parsed.total_additions, 2);
        assert_eq!(parsed.total_deletions, 1);

        let file = &parsed.files[0];
        assert_eq!(file.change_type, FileChangeType::Modified);
        assert_eq!(file.display_path(), "src/lib.rs");
        assert_eq!(file.hunks.len(), 1);
        assert_eq!(file.hunks[0].lines[1].kind, DiffLineKind::Deletion);
        assert_eq!(file.hunks[0].lines[2].kind, DiffLineKind::Addition);
    }

    #[test]
    fn parses_added_file_diff() {
        let diff = "diff --git a/src/new.rs b/src/new.rs\nnew file mode 100644\n--- /dev/null\n+++ b/src/new.rs\n@@ -0,0 +1 @@\n+pub fn new_file() {}\n";

        let parsed = parse_unified_diff(diff).expect("diff should parse");
        let file = &parsed.files[0];

        assert_eq!(file.change_type, FileChangeType::Added);
        assert_eq!(file.display_path(), "src/new.rs");
        assert_eq!(file.additions, 1);
        assert_eq!(file.deletions, 0);
    }
}
