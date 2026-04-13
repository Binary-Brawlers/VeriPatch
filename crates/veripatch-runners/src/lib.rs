//! VeriPatch Runners — check execution backends.
//!
//! Each runner knows how to execute a specific type of check
//! (compile, lint, test, security scan, dependency audit) against
//! a local repository.

pub mod runner;

pub use runner::{
    CheckResult, CheckStatus, ProjectLanguage, RunnerContext, SupportedLanguageInfo,
    detect_project_language, run_default_checks, supported_languages,
};
