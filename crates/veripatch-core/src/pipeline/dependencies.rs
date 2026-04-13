use anyhow::Result;
use serde::Deserialize;
use tokio::process::Command;

use crate::diff::{ChangedFile, DiffLineKind, ParsedDiff};

pub(super) async fn detect_dependency_notes(
    parsed_diff: &ParsedDiff,
    execution_repo_path: &std::path::Path,
) -> Result<Vec<String>> {
    let mut notes = Vec::new();
    let cargo_manifest_files: Vec<_> = parsed_diff
        .files
        .iter()
        .filter(|file| file.display_path().ends_with("Cargo.toml"))
        .collect();
    let has_cargo_lock = parsed_diff
        .files
        .iter()
        .any(|file| file.display_path() == "Cargo.lock");

    if has_cargo_lock && cargo_manifest_files.is_empty() {
        notes.push(
            "`Cargo.lock` changed without a direct `Cargo.toml` manifest change.".to_string(),
        );
    }

    let dependency_changes = cargo_manifest_files
        .iter()
        .flat_map(|file| extract_dependency_changes(file))
        .collect::<Vec<_>>();

    let metadata_packages = if dependency_changes.is_empty() {
        Vec::new()
    } else {
        load_cargo_metadata_packages(execution_repo_path)
            .await
            .unwrap_or_default()
    };

    for change in dependency_changes {
        notes.push(format!(
            "New Rust dependency added in `{}`: `{}`.",
            change.manifest_path, change.name
        ));

        if change.spec.contains("git =") {
            notes.push(format!(
                "Dependency `{}` uses a git source and should be reviewed for provenance and pinning.",
                change.name
            ));
        }

        if change.spec.contains("path =") {
            notes.push(format!(
                "Dependency `{}` uses a local path source and may rely on repository-specific layout assumptions.",
                change.name
            ));
        }

        if change.spec.contains('"') && change.spec.contains("*") {
            notes.push(format!(
                "Dependency `{}` appears to use a wildcard version requirement.",
                change.name
            ));
        }

        if let Some(package) = metadata_packages
            .iter()
            .find(|package| package.name == change.name)
        {
            if let Some(license) = &package.license {
                notes.push(format!(
                    "Dependency `{}` resolves to version `{}` with license `{}`.",
                    package.name, package.version, license
                ));
            } else {
                notes.push(format!(
                    "Dependency `{}` resolves to version `{}` but does not declare a license in Cargo metadata.",
                    package.name, package.version
                ));
            }

            if let Some(source) = &package.source
                && source.starts_with("git+")
            {
                notes.push(format!(
                    "Dependency `{}` resolves from a git source: `{}`.",
                    package.name, source
                ));
            }
        } else {
            notes.push(format!(
                "Dependency `{}` could not be resolved from `cargo metadata`; verify that the patched checkout is still resolvable.",
                change.name
            ));
        }
    }

    Ok(notes)
}

fn extract_dependency_changes(file: &ChangedFile) -> Vec<DependencyChange> {
    let mut current_section: Option<String> = None;
    let mut changes = Vec::new();

    for hunk in &file.hunks {
        for line in &hunk.lines {
            if matches!(line.kind, DiffLineKind::Context | DiffLineKind::Addition)
                && let Some(section) = parse_toml_section(&line.content)
            {
                current_section = Some(section.to_string());
                continue;
            }

            if line.kind == DiffLineKind::Addition
                && current_section
                    .as_deref()
                    .is_some_and(is_dependency_section)
                && let Some((dependency_name, dependency_spec)) =
                    parse_dependency_entry(&line.content)
            {
                changes.push(DependencyChange {
                    manifest_path: file.display_path(),
                    name: dependency_name.to_string(),
                    spec: dependency_spec.to_string(),
                });
            }
        }
    }

    changes
}

async fn load_cargo_metadata_packages(
    execution_repo_path: &std::path::Path,
) -> Result<Vec<CargoMetadataPackage>> {
    let output = Command::new("cargo")
        .args(["metadata", "--format-version", "1", "--locked"])
        .current_dir(execution_repo_path)
        .output()
        .await?;

    if !output.status.success() {
        anyhow::bail!(String::from_utf8_lossy(&output.stderr).trim().to_string());
    }

    let metadata: CargoMetadata = serde_json::from_slice(&output.stdout)?;
    Ok(metadata.packages)
}

fn parse_toml_section(line: &str) -> Option<&str> {
    let trimmed = line.trim();
    trimmed.strip_prefix('[')?.strip_suffix(']')
}

fn is_dependency_section(section: &str) -> bool {
    matches!(
        section,
        "dependencies" | "dev-dependencies" | "build-dependencies" | "workspace.dependencies"
    )
}

fn parse_dependency_entry(line: &str) -> Option<(&str, &str)> {
    let trimmed = line.trim();
    if trimmed.starts_with('#') || !trimmed.contains('=') {
        return None;
    }

    let mut parts = trimmed.splitn(2, '=');
    let name = parts.next()?.trim();
    let spec = parts.next()?.trim();

    if name.is_empty() || spec.is_empty() {
        None
    } else {
        Some((name, spec))
    }
}

#[derive(Debug, Clone)]
struct DependencyChange {
    manifest_path: String,
    name: String,
    spec: String,
}

#[derive(Debug, Deserialize)]
struct CargoMetadata {
    packages: Vec<CargoMetadataPackage>,
}

#[derive(Debug, Deserialize)]
struct CargoMetadataPackage {
    name: String,
    version: String,
    license: Option<String>,
    source: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::{detect_dependency_notes, parse_dependency_entry};
    use crate::diff::{ChangedFile, DiffHunk, DiffLine, DiffLineKind, FileChangeType, ParsedDiff};

    #[tokio::test]
    async fn reports_added_cargo_dependencies() {
        let parsed_diff = ParsedDiff {
            files: vec![ChangedFile {
                old_path: Some("Cargo.toml".to_string()),
                new_path: Some("Cargo.toml".to_string()),
                change_type: FileChangeType::Modified,
                additions: 2,
                deletions: 0,
                hunks: vec![DiffHunk {
                    header: "@@ -1,2 +1,4 @@".to_string(),
                    lines: vec![
                        DiffLine {
                            kind: DiffLineKind::Context,
                            content: "[dependencies]".to_string(),
                            old_line_number: Some(1),
                            new_line_number: Some(1),
                        },
                        DiffLine {
                            kind: DiffLineKind::Addition,
                            content: "serde = \"1\"".to_string(),
                            old_line_number: None,
                            new_line_number: Some(2),
                        },
                    ],
                }],
            }],
            total_additions: 1,
            total_deletions: 0,
        };

        let notes =
            detect_dependency_notes(&parsed_diff, std::path::Path::new("/tmp/does-not-matter"))
                .await
                .expect("dependency scan should succeed");
        assert!(
            notes
                .iter()
                .any(|note| note == "New Rust dependency added in `Cargo.toml`: `serde`.")
        );
    }

    #[test]
    fn parses_dependency_name_and_spec() {
        assert_eq!(
            parse_dependency_entry("serde = { version = \"1\", features = [\"derive\"] }")
                .map(|(name, spec)| (name.to_string(), spec.to_string())),
            Some((
                "serde".to_string(),
                "{ version = \"1\", features = [\"derive\"] }".to_string()
            ))
        );
    }
}
