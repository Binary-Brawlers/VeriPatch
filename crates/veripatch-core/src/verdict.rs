//! Verdict types: Safe, Risky, Broken.

use serde::{Deserialize, Serialize};

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
    pub verdict: Verdict,
    /// Risk score from 0 (safe) to 100 (broken).
    pub score: u8,
    pub checks: Vec<CheckResult>,
    pub warnings: Vec<String>,
    pub assumptions: Vec<String>,
}

/// Result of an individual check.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CheckResult {
    pub name: String,
    pub passed: bool,
    pub message: Option<String>,
}
