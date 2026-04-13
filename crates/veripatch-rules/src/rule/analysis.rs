use super::types::{Assumption, RiskSeverity, RuleFinding, RuleInputLine};

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

        if contains_process_execution(&lower) {
            let shell_execution = contains_shell_execution(&lower, &line.content);
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

        if contains_dynamic_sql(&lower, &line.content) {
            findings.push(finding(
                "dynamic-sql",
                RiskSeverity::High,
                "Added line appears to construct SQL dynamically.",
                line,
            ));
        }

        if contains_unchecked_deserialization(&lower) {
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

        if contains_environment_lookup(&lower) {
            assumptions.push(Assumption {
                message: "Added code assumes specific environment variables are present."
                    .to_string(),
                file_path: Some(line.file_path.clone()),
                line_number: line.line_number,
            });
        }

        if contains_forceful_assumption(&lower, &line.content, &line.file_path) {
            assumptions.push(Assumption {
                message: "Added code assumes a value is always present or a fallible operation always succeeds.".to_string(),
                file_path: Some(line.file_path.clone()),
                line_number: line.line_number,
            });
        }

        if contains_filesystem_assumption(&lower) {
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

fn contains_process_execution(lower: &str) -> bool {
    contains_any(
        lower,
        &[
            "std::process::command",
            "command::new(",
            "exec(",
            "execsync(",
            "execfile(",
            "execfilesync(",
            "spawn(",
            "spawnsync(",
            "fork(",
            "system(",
            "child_process.",
            "deno.command(",
            "bun.spawn(",
        ],
    )
}

fn contains_shell_execution(lower: &str, original: &str) -> bool {
    contains_any(
        lower,
        &[
            "command::new(\"sh\"",
            "command::new(\"bash\"",
            "command::new(\"zsh\"",
            "command::new(\"cmd\"",
            "command::new(\"powershell\"",
            ".arg(\"-c\"",
            "/bin/sh",
            "/bin/bash",
            "exec(",
            "execsync(",
            "shell: true",
            "deno.command(\"sh\"",
            "deno.command(\"bash\"",
            "bun.spawn([\"sh\"",
            "bun.spawn([\"bash\"",
        ],
    ) || original.contains("${SHELL}")
}

fn contains_dynamic_sql(lower: &str, original: &str) -> bool {
    let contains_sql_keyword = contains_any(
        lower,
        &["select ", "insert ", "update ", "delete from", "where "],
    );
    let contains_dynamic_construction = original.contains('+')
        || original.contains("format!(")
        || original.contains("${")
        || original.contains(".concat(");

    contains_sql_keyword && contains_dynamic_construction
}

fn contains_unchecked_deserialization(lower: &str) -> bool {
    contains_any(
        lower,
        &[
            "serde_json::from_str",
            "serde_json::from_slice",
            "yaml::from_str",
            "toml::from_str",
            "pickle.loads",
            "json.parse(",
            "yaml.parse(",
            "yaml.load(",
            "safe_load(",
        ],
    )
}

fn contains_environment_lookup(lower: &str) -> bool {
    contains_any(
        lower,
        &[
            "std::env::var(",
            "env::var(",
            "process.env.",
            "process.env[",
            "import.meta.env.",
            "deno.env.get(",
            "std::env[",
        ],
    )
}

fn contains_forceful_assumption(lower: &str, original: &str, file_path: &str) -> bool {
    lower.contains(".unwrap()")
        || lower.contains(".expect(")
        || (is_typescript_like_path(file_path) && contains_non_null_assertion(original))
}

fn contains_filesystem_assumption(lower: &str) -> bool {
    contains_any(
        lower,
        &[
            "file::open(",
            "fs::",
            "fs.",
            "fs/",
            "readfile(",
            "readfilesync(",
            "writefile(",
            "writefilesync(",
            "appendfile(",
            "appendfilesync(",
        ],
    )
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

fn contains_any(haystack: &str, needles: &[&str]) -> bool {
    needles.iter().any(|needle| haystack.contains(needle))
}

fn is_typescript_like_path(file_path: &str) -> bool {
    matches!(
        file_path.rsplit('.').next(),
        Some("ts" | "tsx" | "mts" | "cts" | "js" | "jsx" | "mjs" | "cjs")
    )
}

fn contains_non_null_assertion(line: &str) -> bool {
    let chars = line.chars().collect::<Vec<_>>();

    for index in 0..chars.len() {
        if chars[index] != '!' {
            continue;
        }

        let previous = previous_non_whitespace(&chars, index);
        let next = next_non_whitespace(&chars, index + 1);

        if matches!(previous, Some('!' | '=' | '<' | '>')) || matches!(next, Some('=')) {
            continue;
        }

        let previous_supports_assertion = previous.is_some_and(|character| {
            character.is_alphanumeric() || "_)]}>\"'`".contains(character)
        });
        let next_supports_assertion = next
            .is_some_and(|character| ".[(;,:?)]}".contains(character) || character.is_whitespace());

        if previous_supports_assertion && next_supports_assertion {
            return true;
        }
    }

    false
}

fn previous_non_whitespace(chars: &[char], until: usize) -> Option<char> {
    chars[..until]
        .iter()
        .rev()
        .find(|character| !character.is_whitespace())
        .copied()
}

fn next_non_whitespace(chars: &[char], from: usize) -> Option<char> {
    chars[from..]
        .iter()
        .find(|character| !character.is_whitespace())
        .copied()
}

#[cfg(test)]
mod tests {
    use super::{analyze_lines, detect_assumptions};
    use crate::rule::{RiskSeverity, RuleInputLine};

    fn line(file_path: &str, content: &str) -> RuleInputLine {
        RuleInputLine {
            file_path: file_path.to_string(),
            line_number: Some(1),
            content: content.to_string(),
        }
    }

    #[test]
    fn detects_typescript_risky_patterns() {
        let lines = vec![
            line("src/auth.ts", "const token = \"secret-token-value\";"),
            line("src/run.ts", "exec(`rm -rf ${target}`);"),
            line(
                "src/query.ts",
                "const sql = `SELECT * FROM users WHERE id = ${userId}`;",
            ),
            line("src/parse.ts", "const parsed = JSON.parse(body);"),
        ];

        let findings = analyze_lines(&lines);

        assert!(
            findings
                .iter()
                .any(|finding| finding.rule_id == "secret-literal")
        );
        assert!(
            findings
                .iter()
                .any(|finding| finding.rule_id == "shell-execution")
        );
        assert!(
            findings
                .iter()
                .any(|finding| finding.rule_id == "dynamic-sql")
        );
        assert!(
            findings
                .iter()
                .any(|finding| finding.rule_id == "deserialization")
        );
        assert!(
            findings
                .iter()
                .any(|finding| finding.severity == RiskSeverity::High)
        );
    }

    #[test]
    fn detects_process_execution_without_shell() {
        let findings = analyze_lines(&[line(
            "src/worker.ts",
            "const child = spawn(\"node\", [\"worker.js\"]);",
        )]);

        assert!(
            findings
                .iter()
                .any(|finding| finding.rule_id == "process-execution")
        );
        assert!(
            !findings
                .iter()
                .any(|finding| finding.rule_id == "shell-execution")
        );
    }

    #[test]
    fn detects_typescript_assumptions() {
        let assumptions = detect_assumptions(&[
            line("src/config.ts", "const apiKey = process.env.API_KEY;"),
            line("src/user.ts", "const id = user!.id;"),
            line(
                "src/fs.ts",
                "const text = await fs.promises.readFile(path, \"utf8\");",
            ),
        ]);

        assert_eq!(assumptions.len(), 3);
        assert!(
            assumptions
                .iter()
                .any(|assumption| assumption.message.contains("environment variables"))
        );
        assert!(
            assumptions
                .iter()
                .any(|assumption| assumption.message.contains("always present"))
        );
        assert!(
            assumptions
                .iter()
                .any(|assumption| assumption.message.contains("filesystem state"))
        );
    }

    #[test]
    fn ignores_regular_negation_when_looking_for_non_null_assertions() {
        let assumptions = detect_assumptions(&[line(
            "src/check.ts",
            "if (!user || user !== currentUser) { return; }",
        )]);

        assert!(assumptions.is_empty());
    }
}
