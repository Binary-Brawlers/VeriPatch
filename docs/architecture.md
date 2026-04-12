# VeriPatch Documentation

## Architecture

VeriPatch is structured as a Cargo workspace with the following crates:

| Crate | Type | Purpose |
|-------|------|---------|
| `veripatch-app` | Binary | GPUI desktop application |
| `veripatch-cli` | Binary | Command-line interface |
| `veripatch-core` | Library | Core verification engine and pipeline |
| `veripatch-ai` | Library | AI provider integration (OpenRouter) |
| `veripatch-runners` | Library | Check runners (compile, lint, test, etc.) |
| `veripatch-rules` | Library | Security and pattern detection rules |
| `veripatch-report` | Library | Report generation (Markdown, JSON) |

## Verification Pipeline

```
Input (diff/patch/working tree)
  → Parse diff
  → Run checks (compile, lint, test, security, deps)
  → Evaluate rules
  → [Optional] AI review via OpenRouter
  → Score and produce verdict (Safe / Risky / Broken)
  → Generate report
```

## Getting Started

See [README.md](../README.md) for build and run instructions.
