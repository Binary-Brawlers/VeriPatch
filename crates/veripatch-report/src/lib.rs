//! VeriPatch Report — report generation.
//!
//! Generates human-readable Markdown reports and machine-readable
//! JSON output from verification results.

pub mod markdown;

pub use markdown::render_markdown;
