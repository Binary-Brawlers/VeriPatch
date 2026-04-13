# AI Output Verifier for Codebases

## Project Summary

AI Output Verifier for Codebases is an open-source desktop application that helps developers verify AI-generated code changes against a real local repository before those changes are trusted, committed, or merged.

It combines local repository checks with optional AI-assisted review, so a developer can validate a change with both deterministic tooling and a configured AI model provider.

Instead of generating more code, this project focuses on answering a more important question:

**Can this AI-generated change be trusted in this specific codebase?**

The tool allows a developer to open a local repository, paste AI-generated code as a diff or import a patch, run a local verification pipeline, optionally ask AI to review the change, and get a clear verdict such as **Safe**, **Risky**, or **Broken**, along with explanations and a report that can be shared in pull requests or reviews.

---

## Why This Project Matters

In 2026, many developers already use AI heavily for coding. The biggest pain point is no longer whether AI can produce code at all. The real issue is that the output is often:

- close but not dependable
- context-blind
- inconsistent with the project
- risky in subtle ways
- difficult to review quickly

This makes a verifier more useful than just another code generator.

This project is especially suitable for Rust because Rust is a strong fit for:

- fast local analysis
- structured diff processing
- low memory usage
- concurrent background tasks
- building reliable desktop tooling

---

## Core Product Vision

Build a desktop application that verifies AI-generated code changes against a real local codebase.

The application should let a developer:

- open a local repository
- paste a unified diff
- import a `.patch` file
- inspect current working tree changes
- load open pull requests from the same project repository
- select a pull request diff as the verification source
- configure an AI model provider and model
- run a verification pipeline locally
- optionally use AI to review and verify the change
- receive a **Safe / Risky / Broken** verdict
- understand why the change is considered safe or risky
- merge or close a reviewed pull request from the app
- export a markdown report for pull requests or code review

This project should feel like a combination of:

- local CI assistant
- smart diff analyzer
- AI skepticism layer
- AI-assisted code reviewer
- pre-merge safety tool

---

## Main User Problem

The problem is not usually that AI writes completely useless code.

The real problem is that AI often writes code that is:

- almost correct
- unaware of hidden project assumptions
- not aligned with conventions or architecture
- weakly tested
- risky in shared or critical paths

This means the tool should not just say whether code compiles.

The real job of the tool is:

**Tell the developer whether this AI-generated change is trustworthy in this specific repository.**

---

## MVP Goal

The MVP should focus on a narrow but useful workflow.

### MVP Objective

Verify a diff against a local repository and return:

- pass/fail results from checks
- impacted-file analysis
- a simple but meaningful risk score
- a human-readable explanation report
- an optional AI review summary based on the diff and local check results

### MVP Inputs

The first version should support:

- pasting a unified diff
- importing a `.patch` file
- reading local uncommitted git changes
- selecting an open pull request from the same project repository

Raw pasted code can be added later, but a diff-first workflow is cleaner and more practical for the first release.

### MVP Outputs

Each verification run should produce:

- overall verdict: **Safe**, **Risky**, or **Broken**
- score from 0 to 100
- failed checks
- warnings
- detected assumptions
- risky patterns identified
- optional AI-generated review notes
- exportable markdown report

---

## MVP Checks

The first version should support the following verification steps.

### 1. Compile / Type Check

Run the codebase type checker or compile check where available.

Examples:

- TypeScript: `tsc --noEmit`
- Rust: `cargo check`

Other languages can be added later.

### 2. Lint

Run the project linter if it exists.

Examples:

- ESLint
- `cargo clippy`

### 3. Test Impacted Files Only

For the MVP, use a practical strategy:

- map changed files
- attempt to identify nearest or related tests
- if uncertain, fall back to running a broader test scope

### 4. Security Checks

Run basic security-focused checks such as:

- accidental secrets in code
- dangerous APIs
- shell execution patterns
- insecure SQL construction
- unsafe deserialization patterns

### 5. Dependency / License Scan

If dependencies changed:

- identify newly added packages or crates
- flag unknown or risky licenses
- flag suspicious dependency additions
- separate lockfile-only changes from direct dependency changes

### 6. Show Assumptions Report

Detect and report hidden assumptions such as:

- required environment variables
- expected response shapes
- implied database schema state
- unhandled null or undefined cases
- missing middleware assumptions
- filesystem or permissions assumptions

### 7. Explain Why a Change Is Risky

Provide short, readable explanations such as:

- change touches authentication flow
- change affects payment logic
- shared utility modified across many files
- tests passed but critical path still weakly covered
- new dependency introduced without surrounding tests

### 8. Optional AI Review

If the developer configures an AI provider, the app can send the diff and selected verification results to the model to produce a review focused on risk, assumptions, and likely regressions.

For the MVP:

- support provider configuration in settings
- support only **OpenRouter** as the provider
- let the developer choose a model exposed through OpenRouter
- keep AI review optional rather than required for a verdict
- use AI for explanation and risk analysis, not automatic code rewriting
- make it clear what repository context is being sent to the model

### 9. Pull Request Actions

For repositories hosted on GitHub, the desktop app should also support a same-project pull request workflow:

- list open pull requests for the currently opened repository
- verify the selected pull request diff against a clean checkout of the project
- allow the developer to either merge or close the pull request after review
- keep these actions explicit and user-driven rather than automatic

---

## What Should Not Be in the MVP

To keep the project focused, the following should not be included in the first version:

- support for many languages at launch
- full semantic analysis for all ecosystems
- support for multiple AI providers at launch
- cloud-hosted repository verification as the primary mode
- AI-based code rewriting or auto-fixing
- deep pull request platform integration
- full execution sandboxing for arbitrary projects

These can be added later as the project grows.

---

## Product Structure

The project should be designed in a way that supports both immediate usability and future extensibility.

### Desktop App

Use a desktop app as the main user-facing product.

Recommended stack:

- **Tauri**
- **Rust backend**
- **React frontend**

Why this fits well:

- Rust handles verification logic efficiently
- Tauri keeps the desktop app lightweight
- React makes the UI easier to build and iterate on

### AI Provider Configuration

The desktop app should include a settings flow where the developer can configure their preferred AI verification provider.

For the first release:

- support only **OpenRouter**
- allow the developer to enter and store their own API key
- allow model selection from the OpenRouter catalog
- treat the provider behind a small abstraction so additional providers can be added later
- keep local verification available even when no AI provider is configured

### Core Engine

The verification engine should be built as a separate Rust crate from the beginning.

This makes it easier to later expose the same functionality through:

- a CLI
- CI integrations
- editor extensions
- GitHub Actions

---

## Recommended Project Structure

A strong modular structure could look like this:

```text
apps/
  desktop/

crates/
  verifier-core/
  verifier-ai/
  verifier-runners/
  verifier-rules/
  verifier-report/

tools/
  cli/

examples/
docs/
