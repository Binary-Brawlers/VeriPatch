//! Rule trait and common types.

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

pub fn analyze_lines(lines: &[RuleInputLine]) -> Vec<RuleFinding> {
    let mut findings = Vec::new();

    for line in lines {
        let lower = line.content.to_ascii_lowercase();

        if looks_like_secret(&lower) {
            findings.push(finding(
                "secret-literal",
                RiskSeverity::High,
                "Added line appears to hardcode a secret or credential.",
                line,
            ));
        }

        if lower.contains("std::process::command")
            || lower.contains("command::new(")
            || lower.contains("exec(")
            || lower.contains("spawn(")
            || lower.contains("system(")
        {
            let shell_execution = lower.contains("command::new(\"sh\"")
                || lower.contains("command::new(\"bash\"")
                || lower.contains("command::new(\"zsh\"")
                || lower.contains("command::new(\"cmd\"")
                || lower.contains("command::new(\"powershell\"")
                || lower.contains(".arg(\"-c\"")
                || lower.contains("/bin/sh")
                || lower.contains("/bin/bash");

            findings.push(finding(
                if shell_execution {
                    "shell-execution"
                } else {
                    "process-execution"
                },
                if shell_execution {
                    RiskSeverity::High
                } else {
                    RiskSeverity::Medium
                },
                if shell_execution {
                    "Added line introduces shell execution."
                } else {
                    "Added line introduces subprocess execution."
                },
                line,
            ));
        }

        if (lower.contains("select ") || lower.contains("insert ") || lower.contains("update "))
            && (line.content.contains('+')
                || line.content.contains("format!(")
                || line.content.contains("${"))
        {
            findings.push(finding(
                "dynamic-sql",
                RiskSeverity::High,
                "Added line appears to construct SQL dynamically.",
                line,
            ));
        }

        if lower.contains("serde_json::from_str")
            || lower.contains("yaml::from_str")
            || lower.contains("pickle.loads")
        {
            findings.push(finding(
                "deserialization",
                RiskSeverity::Medium,
                "Added line introduces unchecked deserialization logic.",
                line,
            ));
        }
    }

    findings
}

pub fn detect_assumptions(lines: &[RuleInputLine]) -> Vec<Assumption> {
    let mut assumptions = Vec::new();

    for line in lines {
        let lower = line.content.to_ascii_lowercase();

        if lower.contains("std::env::var(") || lower.contains("env::var(") {
            assumptions.push(Assumption {
                message: "Added code assumes specific environment variables are present."
                    .to_string(),
                file_path: Some(line.file_path.clone()),
                line_number: line.line_number,
            });
        }

        if lower.contains(".unwrap()") || lower.contains(".expect(") {
            assumptions.push(Assumption {
                message: "Added code assumes a value is always present or a fallible operation always succeeds.".to_string(),
                file_path: Some(line.file_path.clone()),
                line_number: line.line_number,
            });
        }

        if lower.contains("file::open(") || lower.contains("fs::") {
            assumptions.push(Assumption {
                message:
                    "Added code assumes filesystem state or permissions are available at runtime."
                        .to_string(),
                file_path: Some(line.file_path.clone()),
                line_number: line.line_number,
            });
        }
    }

    assumptions
}

fn finding(
    rule_id: &str,
    severity: RiskSeverity,
    message: &str,
    line: &RuleInputLine,
) -> RuleFinding {
    RuleFinding {
        rule_id: rule_id.to_string(),
        severity,
        message: message.to_string(),
        file_path: Some(line.file_path.clone()),
        line_number: line.line_number,
    }
}

fn looks_like_secret(lower: &str) -> bool {
    let secret_keys = ["api_key", "apikey", "secret", "token", "password", "passwd"];

    let has_secret_key = secret_keys.iter().any(|key| lower.contains(key));
    let has_assignment = lower.contains('=') || lower.contains(':');
    let has_literal = lower.contains('"') || lower.contains('\'');

    has_secret_key && has_assignment && has_literal
}
