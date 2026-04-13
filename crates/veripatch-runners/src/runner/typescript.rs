use std::{
    collections::BTreeMap,
    fs,
    path::{Path, PathBuf},
};

use anyhow::Result;
use serde::Deserialize;

use super::{CheckResult, CheckStatus, RunnerContext};
use crate::runner::command::{run_command_check_owned, skipped_check};

pub(super) async fn run_typescript_checks(
    context: &RunnerContext,
    changed_paths: &[String],
) -> Result<Vec<CheckResult>> {
    let manifest = load_package_manifest(&context.repo_path);
    let mut results = Vec::with_capacity(3);

    results.push(run_typescript_typecheck(context, manifest.as_ref()).await?);
    results.push(run_typescript_lint(context, manifest.as_ref()).await?);
    results.push(run_typescript_tests(context, manifest.as_ref(), changed_paths).await?);

    Ok(results)
}

pub(super) fn is_typescript_project(path: &Path, changed_paths: &[String]) -> bool {
    let package_json_path = path.join("package.json");
    if !package_json_path.exists() {
        return false;
    }

    let Some(manifest) = load_package_manifest(path) else {
        return false;
    };

    manifest_declares_typescript(&manifest)
        || has_typescript_config(path)
        || changed_paths
            .iter()
            .any(|changed_path| is_typescript_related_path(changed_path))
}

pub(super) fn is_typescript_related_path(path: &str) -> bool {
    let normalized = path.replace('\\', "/");
    normalized.ends_with(".ts")
        || normalized.ends_with(".tsx")
        || normalized.ends_with(".cts")
        || normalized.ends_with(".mts")
        || normalized.ends_with(".d.ts")
        || normalized == "package.json"
        || normalized.ends_with("/package.json")
        || normalized.starts_with("tsconfig")
        || normalized.contains("/tsconfig")
}

async fn run_typescript_typecheck(
    context: &RunnerContext,
    manifest: Option<&PackageManifest>,
) -> Result<CheckResult> {
    if let Some(spec) = discover_typecheck_command(&context.repo_path, manifest) {
        return run_typescript_command(context, "Compile / Type Check", spec).await;
    }

    Ok(skipped_check(
        "Compile / Type Check",
        "no TypeScript typecheck command was detected in package.json scripts or local tooling",
    ))
}

async fn run_typescript_lint(
    context: &RunnerContext,
    manifest: Option<&PackageManifest>,
) -> Result<CheckResult> {
    if let Some(spec) = discover_lint_command(&context.repo_path, manifest) {
        return run_typescript_command(context, "Lint", spec).await;
    }

    Ok(skipped_check(
        "Lint",
        "no TypeScript lint command was detected in package.json scripts or local tooling",
    ))
}

async fn run_typescript_tests(
    context: &RunnerContext,
    manifest: Option<&PackageManifest>,
    changed_paths: &[String],
) -> Result<CheckResult> {
    let Some(spec) = discover_test_command(&context.repo_path, manifest) else {
        return Ok(skipped_check(
            "Impacted Tests",
            "no TypeScript test command was detected in package.json scripts or local tooling",
        ));
    };

    let mut result = run_typescript_command(context, "Impacted Tests", spec.clone()).await?;
    result.summary = match result.status {
        CheckStatus::Passed => {
            "Ran the configured TypeScript test suite; targeted TypeScript test selection is not implemented yet.".to_string()
        }
        CheckStatus::Failed => format!(
            "The configured TypeScript test suite failed; targeted TypeScript test selection is not implemented yet ({} changed path(s)).",
            changed_paths.len()
        ),
        CheckStatus::Skipped => unreachable!("TypeScript tests are either run or skipped earlier"),
    };
    Ok(result)
}

async fn run_typescript_command(
    context: &RunnerContext,
    name: &str,
    spec: CommandSpec,
) -> Result<CheckResult> {
    let mut result = run_command_check_owned(context, name, &spec.program, &spec.args).await?;
    result.command = Some(spec.display);
    Ok(result)
}

fn discover_typecheck_command(
    repo_path: &Path,
    manifest: Option<&PackageManifest>,
) -> Option<CommandSpec> {
    for script_name in ["typecheck", "check-types", "types", "tsc"] {
        if manifest
            .and_then(|manifest| manifest.scripts.get(script_name))
            .is_some()
        {
            return Some(script_command(repo_path, script_name));
        }
    }

    let local_tsc = find_local_bin(repo_path, "tsc")?;
    Some(CommandSpec {
        program: local_tsc,
        args: vec!["--noEmit".to_string()],
        display: "tsc --noEmit".to_string(),
    })
}

fn discover_lint_command(
    repo_path: &Path,
    manifest: Option<&PackageManifest>,
) -> Option<CommandSpec> {
    for script_name in ["lint", "eslint"] {
        if manifest
            .and_then(|manifest| manifest.scripts.get(script_name))
            .is_some()
        {
            return Some(script_command(repo_path, script_name));
        }
    }

    let local_eslint = find_local_bin(repo_path, "eslint")?;
    Some(CommandSpec {
        program: local_eslint,
        args: vec![".".to_string()],
        display: "eslint .".to_string(),
    })
}

fn discover_test_command(
    repo_path: &Path,
    manifest: Option<&PackageManifest>,
) -> Option<CommandSpec> {
    if let Some(manifest) = manifest {
        if let Some(script) = manifest.scripts.get("test")
            && !is_placeholder_test_script(script)
        {
            return Some(script_command(repo_path, "test"));
        }
    }

    if let Some(vitest) = find_local_bin(repo_path, "vitest") {
        return Some(CommandSpec {
            program: vitest,
            args: vec!["run".to_string()],
            display: "vitest run".to_string(),
        });
    }

    if let Some(jest) = find_local_bin(repo_path, "jest") {
        return Some(CommandSpec {
            program: jest,
            args: vec!["--runInBand".to_string()],
            display: "jest --runInBand".to_string(),
        });
    }

    None
}

fn script_command(repo_path: &Path, script_name: &str) -> CommandSpec {
    match detect_package_manager(repo_path) {
        PackageManager::Pnpm => CommandSpec {
            program: PathBuf::from("pnpm"),
            args: vec!["run".to_string(), script_name.to_string()],
            display: format!("pnpm run {script_name}"),
        },
        PackageManager::Yarn => CommandSpec {
            program: PathBuf::from("yarn"),
            args: vec![script_name.to_string()],
            display: format!("yarn {script_name}"),
        },
        PackageManager::Bun => CommandSpec {
            program: PathBuf::from("bun"),
            args: vec!["run".to_string(), script_name.to_string()],
            display: format!("bun run {script_name}"),
        },
        PackageManager::Npm => CommandSpec {
            program: PathBuf::from("npm"),
            args: vec!["run".to_string(), script_name.to_string()],
            display: format!("npm run {script_name}"),
        },
    }
}

fn detect_package_manager(repo_path: &Path) -> PackageManager {
    if repo_path.join("pnpm-lock.yaml").exists() {
        PackageManager::Pnpm
    } else if repo_path.join("yarn.lock").exists() {
        PackageManager::Yarn
    } else if repo_path.join("bun.lock").exists() || repo_path.join("bun.lockb").exists() {
        PackageManager::Bun
    } else {
        PackageManager::Npm
    }
}

fn find_local_bin(repo_path: &Path, name: &str) -> Option<PathBuf> {
    let bin_dir = repo_path.join("node_modules").join(".bin");
    [
        bin_dir.join(name),
        bin_dir.join(format!("{name}.cmd")),
        bin_dir.join(format!("{name}.ps1")),
    ]
    .into_iter()
    .find(|candidate| candidate.exists())
}

fn has_typescript_config(repo_path: &Path) -> bool {
    fs::read_dir(repo_path)
        .ok()
        .into_iter()
        .flatten()
        .filter_map(Result::ok)
        .map(|entry| entry.file_name())
        .filter_map(|name| name.into_string().ok())
        .any(|name| name.starts_with("tsconfig") && name.ends_with(".json"))
}

fn load_package_manifest(repo_path: &Path) -> Option<PackageManifest> {
    let raw = fs::read_to_string(repo_path.join("package.json")).ok()?;
    serde_json::from_str(&raw).ok()
}

fn manifest_declares_typescript(manifest: &PackageManifest) -> bool {
    manifest.dependencies.contains_key("typescript")
        || manifest.dev_dependencies.contains_key("typescript")
        || manifest.optional_dependencies.contains_key("typescript")
}

fn is_placeholder_test_script(script: &str) -> bool {
    let normalized = script.to_ascii_lowercase();
    normalized.contains("no test specified") || normalized.contains("exit 1")
}

#[derive(Debug, Clone)]
struct CommandSpec {
    program: PathBuf,
    args: Vec<String>,
    display: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum PackageManager {
    Npm,
    Pnpm,
    Yarn,
    Bun,
}

#[derive(Debug, Clone, Deserialize, Default)]
struct PackageManifest {
    #[serde(default)]
    scripts: BTreeMap<String, String>,
    #[serde(default)]
    dependencies: BTreeMap<String, String>,
    #[serde(rename = "devDependencies", default)]
    dev_dependencies: BTreeMap<String, String>,
    #[serde(rename = "optionalDependencies", default)]
    optional_dependencies: BTreeMap<String, String>,
}

#[cfg(test)]
mod tests {
    use super::{
        detect_package_manager, discover_test_command, is_placeholder_test_script,
        is_typescript_project, is_typescript_related_path,
    };
    use std::fs;

    #[test]
    fn detects_typescript_project_from_manifest_and_config() {
        let root = tempfile::tempdir().expect("temp dir");
        fs::write(
            root.path().join("package.json"),
            r#"{"devDependencies":{"typescript":"^5.0.0"}}"#,
        )
        .expect("write package.json");
        fs::write(root.path().join("tsconfig.json"), "{}").expect("write tsconfig.json");

        assert!(is_typescript_project(root.path(), &[]));
    }

    #[test]
    fn does_not_treat_plain_javascript_package_as_typescript() {
        let root = tempfile::tempdir().expect("temp dir");
        fs::write(root.path().join("package.json"), r#"{"name":"demo"}"#)
            .expect("write package.json");

        assert!(!is_typescript_project(
            root.path(),
            &["src/index.js".to_string()]
        ));
    }

    #[test]
    fn recognizes_typescript_related_paths() {
        assert!(is_typescript_related_path("src/index.ts"));
        assert!(is_typescript_related_path("tsconfig.base.json"));
        assert!(!is_typescript_related_path("src/index.rs"));
    }

    #[test]
    fn ignores_placeholder_test_script() {
        assert!(is_placeholder_test_script(
            "echo \"Error: no test specified\" && exit 1"
        ));
    }

    #[test]
    fn chooses_package_manager_from_lockfile() {
        let root = tempfile::tempdir().expect("temp dir");
        fs::write(root.path().join("pnpm-lock.yaml"), "lockfileVersion: '9.0'")
            .expect("write pnpm lockfile");

        assert_eq!(
            detect_package_manager(root.path()),
            super::PackageManager::Pnpm
        );
    }

    #[test]
    fn falls_back_to_vitest_when_test_script_is_placeholder() {
        let root = tempfile::tempdir().expect("temp dir");
        fs::write(
            root.path().join("package.json"),
            r#"{"scripts":{"test":"echo \"Error: no test specified\" && exit 1"}}"#,
        )
        .expect("write package.json");
        let bin_dir = root.path().join("node_modules").join(".bin");
        fs::create_dir_all(&bin_dir).expect("create bin dir");
        fs::write(bin_dir.join("vitest"), "").expect("write vitest binary");

        let manifest = super::load_package_manifest(root.path()).expect("load package manifest");
        let command =
            discover_test_command(root.path(), Some(&manifest)).expect("discover test command");

        assert_eq!(command.display, "vitest run");
    }
}
