# Implementation plan

## Milestone 1 — Foundation
- Outcome: the project runs, reads configuration, and fetches one feed.
- Tasks: run `cargo new`, configure fmt and clippy, add `config.rs`, `domain.rs`, and `error.rs`, and implement `FeedSource` with RSS support for BBC.
- Acceptance checks: `cargo test` passes; the BBC fetch logs a list of titles (use a fixture in tests).
- Risk: the feed URL may be wrong. Verify it manually.

## Milestone 2 — Pipeline
- Outcome: raw items become classified articles without duplicates.
- Tasks: implement `normalize`, `dedup`, and `classify`; add keywords to configuration and complete the unit tests.
- Acceptance checks: tests cover all three categories, ambiguous cases, and duplicate URLs with tracking parameters.
- Risk: keyword rules may be too broad or too narrow. Prepare a set of real example headlines.

## Milestone 3 — Storage + scheduler
- Outcome: articles are stored and updated automatically.
- Tasks: add the SQLite schema and migrations, repository, per-source scheduler, and ETag/Last-Modified storage.
- Acceptance checks: fetching the same feed twice creates no duplicates; when one source is deliberately made to fail, the others still run.

## Milestone 4 — Web
- Outcome: stories can be read in a browser.
- Tasks: add routes `/`, `/category/{slug}`, `/source/{slug}`, templates with askama, and `/health`.
- Acceptance checks: category and source filters work; every item links to its original source.

## Milestone 5 — Add sources and hardening
- Outcome: every source in the PRD is handled, or intentionally skipped with a recorded reason.
- Tasks: decide how to handle Reuters and AP; add The Guardian, DW, Al Jazeera, NYT, and CNN; add retries and timeouts; complete the README.
- Acceptance checks: `last_error` is visible for failed sources; the README can be followed from a clean start.

## Phase 2 (not scheduled)
LLM classifier, summaries, search, and notifications.
