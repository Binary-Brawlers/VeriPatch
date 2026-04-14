//! Runner trait and common types.

mod command;
mod rust;
mod typescript;

use std::path::Path;
use std::path::PathBuf;

use anyhow::Result;
use serde::{Deserialize, Serialize};

use command::skipped_check;

#[derive(Debug, Clone)]
pub struct RunnerContext {
    pub repo_path: PathBuf,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProjectLanguage {
    Rust,
    TypeScript,
    Unsupported,
}

impl ProjectLanguage {
    pub fn display_name(self) -> &'static str {
        match self {
            Self::Rust => "Rust",
            Self::TypeScript => "TypeScript",
            Self::Unsupported => "Unsupported",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SupportedLanguageInfo {
    pub id: ProjectLanguage,
    pub name: String,
    pub manifests: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CheckStatus {
    Passed,
    Failed,
    Skipped,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CheckResult {
    pub name: String,
    pub status: CheckStatus,
    pub command: Option<String>,
    pub summary: String,
    pub details: Option<String>,
}

pub async fn run_default_checks(
    context: &RunnerContext,
    changed_paths: &[String],
) -> Result<Vec<CheckResult>> {
    if let Some(project_root) = detect_supported_project(&context.repo_path, changed_paths) {
        return match project_root {
            SupportedProject {
                language: ProjectLanguage::Rust,
                root: rust_root,
            } => {
                rust::run_rust_checks(
                    &RunnerContext {
                        repo_path: rust_root,
                    },
                    changed_paths,
                )
                .await
            }
            SupportedProject {
                language: ProjectLanguage::TypeScript,
                root: typescript_root,
            } => {
                typescript::run_typescript_checks(
                    &RunnerContext {
                        repo_path: typescript_root,
                    },
                    changed_paths,
                )
                .await
            }
            SupportedProject {
                language: ProjectLanguage::Unsupported,
                ..
            } => unreachable!("unsupported projects are filtered out before checks run"),
        };
    }

    let skip_reason = format!(
        "no supported Rust or TypeScript project root was detected from `{}`; select a folder that contains `Cargo.toml` or a TypeScript-aware `package.json`",
        context.repo_path.display()
    );

    Ok(vec![
        skipped_check("Compile / Type Check", &skip_reason),
        skipped_check("Lint", &skip_reason),
        skipped_check("Impacted Tests", &skip_reason),
    ])
}

pub fn supported_languages() -> Vec<SupportedLanguageInfo> {
    vec![
        SupportedLanguageInfo {
            id: ProjectLanguage::Rust,
            name: ProjectLanguage::Rust.display_name().to_string(),
            manifests: vec!["Cargo.toml".to_string()],
        },
        SupportedLanguageInfo {
            id: ProjectLanguage::TypeScript,
            name: ProjectLanguage::TypeScript.display_name().to_string(),
            manifests: vec!["package.json".to_string(), "tsconfig.json".to_string()],
        },
    ]
}

pub fn detect_project_language(path: &Path) -> ProjectLanguage {
    detect_supported_project(path, &[])
        .map(|project| project.language)
        .unwrap_or(ProjectLanguage::Unsupported)
}

fn detect_supported_project(path: &Path, changed_paths: &[String]) -> Option<SupportedProject> {
    let mut current = Some(path);
    let changed_paths_look_typescript = changed_paths
        .iter()
        .any(|changed_path| typescript::is_typescript_related_path(changed_path));

    while let Some(dir) = current {
        let rust_manifest = dir.join("Cargo.toml");
        let package_manifest = dir.join("package.json");

        if package_manifest.exists()
            && typescript::is_typescript_project(dir, changed_paths)
            && (!rust_manifest.exists() || changed_paths_look_typescript)
        {
            return Some(SupportedProject {
                language: ProjectLanguage::TypeScript,
                root: dir.to_path_buf(),
            });
        }

        if rust_manifest.exists() {
            return Some(SupportedProject {
                language: ProjectLanguage::Rust,
                root: dir.to_path_buf(),
            });
        }

        current = dir.parent();
    }

    None
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct SupportedProject {
    language: ProjectLanguage,
    root: PathBuf,
}

#[cfg(test)]
mod tests {
    use super::{
        ProjectLanguage, SupportedProject, detect_project_language, detect_supported_project,
        supported_languages,
    };
    use std::fs;

    #[test]
    fn finds_rust_manifest_in_ancestor_directory() {
        let root = tempfile::tempdir().expect("temp dir");
        fs::write(
            root.path().join("Cargo.toml"),
            "[package]\nname = \"demo\"\nversion = \"0.1.0\"\n",
        )
        .expect("write Cargo.toml");

        let nested = root.path().join("crates").join("demo").join("src");
        fs::create_dir_all(&nested).expect("create nested path");

        assert_eq!(
            detect_supported_project(&nested, &[]),
            Some(SupportedProject {
                language: ProjectLanguage::Rust,
                root: root.path().to_path_buf(),
            })
        );
    }

    #[test]
    fn finds_typescript_manifest_in_ancestor_directory() {
        let root = tempfile::tempdir().expect("temp dir");
        fs::write(
            root.path().join("package.json"),
            r#"{"devDependencies":{"typescript":"^5.0.0"}}"#,
        )
        .expect("write package.json");
        fs::write(root.path().join("tsconfig.json"), "{}").expect("write tsconfig.json");

        let nested = root.path().join("src").join("components");
        fs::create_dir_all(&nested).expect("create nested path");

        assert_eq!(
            detect_supported_project(&nested, &["src/index.ts".to_string()]),
            Some(SupportedProject {
                language: ProjectLanguage::TypeScript,
                root: root.path().to_path_buf(),
            })
        );
    }

    #[test]
    fn prefers_typescript_when_both_manifests_exist_and_diff_is_typescript() {
        let root = tempfile::tempdir().expect("temp dir");
        fs::write(
            root.path().join("Cargo.toml"),
            "[package]\nname = \"demo\"\nversion = \"0.1.0\"\n",
        )
        .expect("write Cargo.toml");
        fs::write(
            root.path().join("package.json"),
            r#"{"devDependencies":{"typescript":"^5.0.0"}}"#,
        )
        .expect("write package.json");
        fs::write(root.path().join("tsconfig.json"), "{}").expect("write tsconfig.json");

        assert_eq!(
            detect_supported_project(root.path(), &["frontend/app.tsx".to_string()]),
            Some(SupportedProject {
                language: ProjectLanguage::TypeScript,
                root: root.path().to_path_buf(),
            })
        );
    }

    #[test]
    fn returns_none_when_no_supported_manifest_exists() {
        let root = tempfile::tempdir().expect("temp dir");
        let nested = root.path().join("plain").join("folder");
        fs::create_dir_all(&nested).expect("create nested path");

        assert_eq!(detect_supported_project(&nested, &[]), None);
        assert_eq!(
            detect_project_language(&nested),
            ProjectLanguage::Unsupported
        );
    }

    #[test]
    fn exposes_supported_languages_registry() {
        let languages = supported_languages();

        assert_eq!(languages.len(), 2);
        assert_eq!(languages[0].id, ProjectLanguage::Rust);
        assert_eq!(languages[1].id, ProjectLanguage::TypeScript);
    }
}
