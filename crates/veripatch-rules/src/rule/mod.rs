//! Rule trait and common types.

mod analysis;
mod types;

pub use analysis::{analyze_lines, detect_assumptions};
pub use types::{Assumption, RiskSeverity, RuleFinding, RuleInputLine};
