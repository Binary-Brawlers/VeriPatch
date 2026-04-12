//! VeriPatch Core — the verification engine.
//!
//! This crate contains the core pipeline that coordinates diff parsing,
//! check execution, rule evaluation, and report generation.

pub mod diff;
pub mod pipeline;
pub mod verdict;
