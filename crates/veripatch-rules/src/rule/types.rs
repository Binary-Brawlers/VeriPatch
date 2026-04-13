use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuleInputLine {
    pub file_path: String,
    pub line_number: Option<usize>,
    pub content: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RiskSeverity {
    Low,
    Medium,
    High,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuleFinding {
    pub rule_id: String,
    pub severity: RiskSeverity,
    pub message: String,
    pub file_path: Option<String>,
    pub line_number: Option<usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Assumption {
    pub message: String,
    pub file_path: Option<String>,
    pub line_number: Option<usize>,
}
