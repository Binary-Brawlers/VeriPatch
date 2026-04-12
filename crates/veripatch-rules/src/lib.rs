//! VeriPatch Rules — security and pattern detection.
//!
//! Defines rules for detecting risky patterns such as secrets in code,
//! dangerous API usage, shell execution, insecure SQL construction,
//! and unsafe deserialization.

pub mod rule;
