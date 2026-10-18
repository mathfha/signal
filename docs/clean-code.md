# Rust clean code guidelines

## Principles
- Keep functions small and focused. If a comment is needed to explain what a function does, rename it or split it up.
- Separate pure logic from I/O. Keep I/O at the edges (`sources`, `storage`, `web`) and logic in the middle (`pipeline`).
- Make dependencies point inward: `domain` should not depend on `web` or `storage`.
- Use clear names such as `normalize_url` instead of `process`. Prefix booleans with `is_` or `has_`.

## Error handling
- In library code, use `Result<T, AppError>` with `thiserror`. Use `anyhow` only in `main`.
- Do not use `unwrap()` or `expect()` outside tests. Use `?`.
- Add context to errors, such as the source or URL involved. Error messages should help with debugging.
- If one source fails, log the error and continue; do not panic.

## Ownership and types
- Use `&str` or `&[T]` for function parameters instead of `String` or `Vec<T>`, unless ownership is needed.
- Do not add `.clone()` just to satisfy the compiler. Consider who should own the data first.
- Use domain types such as the `Category` enum and `SourceId` instead of raw strings everywhere.
- Make `match` expressions exhaustive. Avoid `_ =>` while an enum may gain variants.

## Code structure
- Keep `main.rs` limited to wiring: read configuration, build components, and run the application.
- Give each module one responsibility. A file over 300 lines is usually a sign that it should be split.
- Keep public functions to a minimum. Default to `pub(crate)` or private.
- Do not mix business logic into async code. Call pure functions from async code instead.

## Testing
- Unit-test every `pipeline` function (`normalize`, `dedup`, `classify`), including ambiguous cases.
- Use real RSS fixtures in `tests/fixtures/` to test the parser. Do not access the network in tests.
- Test storage with an in-memory SQLite database.
- Name tests after the behavior, for example `classifies_ransomware_story_as_cybercrime`.

## Logging
- Use `tracing`. Use `info` for normal flow, `warn` when a source fails, and `error` for issues that need attention.
- Do not log sensitive data or full article text.

## Style and tooling
- These checks must pass:
  - `cargo fmt --check`
  - `cargo clippy --all-targets -- -D warnings`
  - `cargo test`
- Comments should explain why, not what.
- Use named constants instead of magic numbers, such as `FETCH_TIMEOUT` and `MAX_SUMMARY_CHARS`.
- Keep commits small and messages clear, for example `feat: add guardian rss source`.

## Diff review checklist
- [ ] Were any new `unwrap()` or `expect()` calls added?
- [ ] Is any `.clone()` unnecessary?
- [ ] Are new logic paths covered by tests?
- [ ] Was a dependency added? Is there a reason?
- [ ] Were the docs updated?
