//! VeriPatch Runners — check execution backends.
//!
//! Each runner knows how to execute a specific type of check
//! (compile, lint, test, security scan, dependency audit) against
//! a local repository.

pub mod runner;
