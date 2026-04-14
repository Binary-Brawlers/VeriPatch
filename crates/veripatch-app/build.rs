use std::env;
use std::fs;
use std::path::Path;
use std::process::Command;

fn main() {
    let manifest_dir =
        env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR is required for veripatch-app");
    let frontend_dir = Path::new(&manifest_dir).join("frontend");

    println!(
        "cargo:rerun-if-changed={}",
        frontend_dir.join("package.json").display()
    );
    println!(
        "cargo:rerun-if-changed={}",
        frontend_dir.join("vite.config.js").display()
    );
    println!(
        "cargo:rerun-if-changed={}",
        frontend_dir.join("index.html").display()
    );
    println!(
        "cargo:rerun-if-changed={}",
        frontend_dir.join("styles.css").display()
    );
    println!(
        "cargo:rerun-if-changed={}",
        frontend_dir.join("src").display()
    );

    if env::var_os("CARGO_FEATURE_DESKTOP_UI").is_some() {
        build_frontend(&frontend_dir);
    }

    tauri_build::build();
}

fn build_frontend(frontend_dir: &Path) {
    ensure_frontend_dependencies(frontend_dir);

    let npm = if cfg!(windows) { "npm.cmd" } else { "npm" };
    let status = Command::new(npm)
        .args(["run", "build"])
        .current_dir(frontend_dir)
        .status()
        .unwrap_or_else(|error| {
            panic!(
                "failed to start `{npm} run build` in `{}`: {error}. \
                 Install Node.js dependencies in the frontend directory first.",
                frontend_dir.display()
            )
        });

    if !status.success() {
        panic!(
            "frontend build failed in `{}`. Run `npm install` there and retry.",
            frontend_dir.display()
        );
    }
}

fn ensure_frontend_dependencies(frontend_dir: &Path) {
    let vite_binary = frontend_dir
        .join("node_modules")
        .join(".bin")
        .join(if cfg!(windows) { "vite.cmd" } else { "vite" });

    if vite_binary.exists() {
        return;
    }

    let npm = if cfg!(windows) { "npm.cmd" } else { "npm" };

    if run_npm_install(frontend_dir, npm) {
        return;
    }

    // Recover from partially updated dependency trees (for example npm ENOTEMPTY).
    let node_modules_dir = frontend_dir.join("node_modules");
    if node_modules_dir.exists() {
        fs::remove_dir_all(&node_modules_dir).unwrap_or_else(|error| {
            panic!(
                "frontend dependency install failed in `{}` and cleanup of `{}` failed: {error}",
                frontend_dir.display(),
                node_modules_dir.display()
            )
        });
    }

    if !run_npm_install(frontend_dir, npm) {
        panic!(
            "frontend dependency install failed in `{}` after retry",
            frontend_dir.display()
        );
    }
}

fn run_npm_install(frontend_dir: &Path, npm: &str) -> bool {
    let status = Command::new(npm)
        .arg("install")
        .current_dir(frontend_dir)
        .status()
        .unwrap_or_else(|error| {
            panic!(
                "failed to start `{npm} install` in `{}`: {error}",
                frontend_dir.display()
            )
        });

    status.success()
}
