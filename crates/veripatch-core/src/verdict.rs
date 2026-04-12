//! Verdict types: Safe, Risky, Broken.

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

use crate::diff::ParsedDiff;
use veripatch_rules::rule::{Assumption, RuleFinding};
use veripatch_runners::runner::CheckResult;

/// The overall verification verdict.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Verdict {
    Safe,
    Risky,
    Broken,
}

/// A scored verification result.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerificationResult {
    pub repo_path: PathBuf,
    pub diff: ParsedDiff,
    pub verdict: Verdict,
    /// Risk score from 0 (safe) to 100 (broken).
    pub score: u8,
    pub checks: Vec<CheckResult>,
    pub warnings: Vec<String>,
    pub assumptions: Vec<Assumption>,
    pub dependency_notes: Vec<String>,
    pub risky_patterns: Vec<RuleFinding>,
}
