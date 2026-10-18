# Instructions for coding agents

## Before changing code
- Read `README.md`, `docs/prd.md`, and `docs/overview.md`.
- Check the existing code before assuming a feature is missing.
- If something is unclear, check the PRD's "Open questions" section. Do not guess.

## Project rules
- Language: Rust (edition 2021). Follow `docs/clean-code.md`.
- Each change should have one goal. Do not add dependencies without a clear reason.
- Do not use `unwrap()` or `expect()` in non-test code.
- Do not store secrets, API keys, or personal data in code or documentation.
- Do not store full article text. Store only the title, short summary, link, source, and publication time.

## Required verification
- `cargo fmt --check`
- `cargo clippy --all-targets -- -D warnings`
- `cargo test`
- If a command cannot run, report which command and why.

## Documentation
- Update documentation in the same change when behavior, source configuration, or the database schema changes.
