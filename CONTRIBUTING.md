# Contributing to VeriPatch

Thank you for your interest in contributing to VeriPatch! This document provides guidelines for contributing to the project.

## Getting Started

1. Fork the repository
2. Clone your fork: `git clone https://github.com/YOUR_USERNAME/veripatch.git`
3. Create a branch: `git checkout -b my-feature`
4. Make your changes
5. Run tests: `cargo test --workspace`
6. Run lints: `cargo clippy --workspace -- -D warnings`
7. Format code: `cargo fmt --all`
8. Commit your changes
9. Push and open a pull request

## Development Setup

### Prerequisites

- Rust 1.85+ (install via [rustup](https://rustup.rs/))
- Git

### Building

```sh
cargo build --workspace
```

### Running Tests

```sh
cargo test --workspace
```

### Running Lints

```sh
cargo clippy --workspace -- -D warnings
cargo fmt --all -- --check
```

## Code Style

- Follow standard Rust conventions
- Run `cargo fmt` before committing
- Ensure `cargo clippy` passes with no warnings
- Write tests for new functionality
- Keep commits focused and atomic

## Pull Request Process

1. Update documentation if your change affects public APIs or behavior
2. Add tests for new functionality
3. Ensure all CI checks pass
4. Request review from a maintainer
5. Squash commits if requested

## Reporting Issues

- Use GitHub Issues to report bugs or request features
- Use the provided issue templates when available
- Include reproduction steps for bugs
- Check existing issues before creating a new one

## Code of Conduct

This project follows the [Contributor Covenant Code of Conduct](CODE_OF_CONDUCT.md). By participating, you agree to uphold this code.
