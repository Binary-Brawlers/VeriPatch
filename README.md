# VeriPatch

**AI Output Verifier for Codebases**

VeriPatch is an open-source desktop application that helps developers verify AI-generated code changes against a real local repository before those changes are trusted, committed, or merged.

Instead of generating more code, VeriPatch answers a more important question:

> **Can this AI-generated change be trusted in this specific codebase?**

## Features

- Open a local repository and paste a unified diff, import a `.patch` file, inspect uncommitted changes, or review same-repo pull requests
- Run a local verification pipeline (compile check, lint, tests, security scan, dependency audit)
- Optionally use an AI model (via OpenRouter) for risk analysis and review
- Get a clear **Safe / Risky / Broken** verdict with explanations
- Merge or close an open pull request after review from the desktop app
- Export markdown reports for pull requests or code review

## Status

> **Early development** — not yet ready for production use.

## Getting Started

### Prerequisites

- [Rust](https://rustup.rs/) (1.85+)
- [Node.js](https://nodejs.org/) (20+)
- macOS, Linux, or Windows
- Optional: [GitHub CLI](https://cli.github.com/) (`gh`) for pull request review, merge, and close actions

### Build

```sh
npm install --prefix crates/veripatch-app/frontend
cargo build --release
```

### Run the Desktop App

```sh
cargo run -p veripatch-app
```

### Run the CLI

```sh
cargo run -p veripatch-cli -- --help
```

## Project Structure

```
crates/
  veripatch-app/        # Tauri desktop application with a React frontend
  veripatch-cli/        # Command-line interface
  veripatch-core/       # Core verification engine
  veripatch-ai/         # AI provider integration (OpenRouter)
  veripatch-runners/    # Check runners (compile, lint, test, etc.)
  veripatch-rules/      # Security & pattern detection rules
  veripatch-report/     # Report generation (markdown, JSON)
docs/                   # Documentation
examples/               # Example diffs and usage
```

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines.

## License

Licensed under either of:

- [Apache License, Version 2.0](LICENSE-APACHE)
- [MIT License](LICENSE-MIT)

at your option.

## Security

See [SECURITY.md](SECURITY.md) for reporting vulnerabilities.
