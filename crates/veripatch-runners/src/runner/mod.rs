//! Runner trait and common types.

mod command;
mod rust;

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
    if let Some(rust_root) = find_rust_manifest_root(&context.repo_path) {
        return rust::run_rust_checks(
            &RunnerContext {
                repo_path: rust_root,
            },
            changed_paths,
        )
        .await;
    }

    let skip_reason = format!(
        "no supported project manifest was detected for the MVP runners from `{}`; for Rust projects, select the repository or workspace folder that contains `Cargo.toml`",
        context.repo_path.display()
    );

    Ok(vec![
        skipped_check("Compile / Type Check", &skip_reason),
        skipped_check("Lint", &skip_reason),
        skipped_check("Impacted Tests", &skip_reason),
    ])
}

fn find_rust_manifest_root(path: &Path) -> Option<PathBuf> {
    let mut current = Some(path);

    while let Some(dir) = current {
        if dir.join("Cargo.toml").exists() {
            return Some(dir.to_path_buf());
        }
        current = dir.parent();
    }

    None
}

#[cfg(test)]
mod tests {
    use super::find_rust_manifest_root;
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
            find_rust_manifest_root(&nested),
            Some(root.path().to_path_buf())
        );
    }

    #[test]
    fn returns_none_when_no_rust_manifest_exists() {
        let root = tempfile::tempdir().expect("temp dir");
        let nested = root.path().join("plain").join("folder");
        fs::create_dir_all(&nested).expect("create nested path");

        assert_eq!(find_rust_manifest_root(&nested), None);
    }
}
