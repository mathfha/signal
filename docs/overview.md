# Architecture overview

## System context
A single Rust binary will run a scheduled fetcher and a web server over SQLite. The current implementation fetches configured feeds once at startup and serves an in-memory snapshot; the scheduler and SQLite repository remain planned work.

```text
 RSS feeds (BBC, Guardian, DW, ...)
        |  HTTP (reqwest, conditional GET)
        v
 +-------------+   Vec<RawItem>   +-----------+   Article   +----------+
 |  Fetcher    | ---------------> | Pipeline  | ----------> | Storage  |
 | (tokio task)|                  | normalize |             | (SQLite) |
 +-------------+                  | dedup     |             +----+-----+
                                  | classify  |                  |
                                  +-----------+                  | query
                                                                 v
                                                          +-------------+
                                          Browser <------ |  Web (axum) |
                                                          +-------------+
```

## Components and responsibilities
| Module | Responsibility |
|---|---|
| `config` | Read `sources.toml` and environment variables. Validate them at startup. |
| `sources` | Define the `FeedSource` trait and RSS/Atom implementations. Fetch and parse one feed. |
| `pipeline` | Normalize, deduplicate, and classify items using pure functions that are easy to test. |
| `storage` | Provide the repository for storing and retrieving `Article` records. Keep all SQL here. |
| `scheduler` | Run a tokio loop that fetches on an interval, handling each source independently. |
| `web` | Define Axum routes for the news index and `/health`; render pages and HTML fragments with Askama for HTMX interactions. |
| `domain` | Define core types: `Article`, `Category`, and `SourceId`. Do not depend on external modules. |
| `error` | Define application error types with `thiserror`. |

## Folder structure
```text
src/
├── main.rs            # wiring saja, tanpa logika
├── domain.rs
├── error.rs
├── config.rs
├── sources/{mod.rs, rss.rs}
├── pipeline/{mod.rs, normalize.rs, dedup.rs, classify.rs}
├── storage/{mod.rs, sqlite.rs}
├── scheduler.rs
└── web/{mod.rs, routes.rs, templates/}  # Askama pages and fragments for HTMX
config/sources.toml
config/categories.toml
tests/
```

## Data flow
1. The scheduler fetches from each source concurrently, with a concurrency limit.
2. `FeedSource::fetch` returns `Vec<RawItem>`. The fetcher stores the feed's ETag and Last-Modified values.
3. `normalize` removes HTML from summaries, shortens summaries, removes tracking parameters from URLs, and converts timestamps to UTC.
4. `storage` checks the database for a matching `url_hash`; the pure `dedup` logic then checks for similar titles.
5. `pipeline::classify` matches configured, token-boundary keywords in the title and summary, then assigns categories. Discard items that match no category.
6. `storage::upsert` stores the resulting items.
7. `web` reads from storage and renders full pages or HTML fragments with
   Askama. HTMX uses the fragments for category and source filters without a
   separate frontend build.

## Current web behavior
- The app fetches each configured source at startup, records per-source
  `last_error` state in memory, and serves the resulting story snapshot.
- The home page sorts stories by publication time. Category and source filters
  use query parameters; full HTML is returned for normal requests and the
  `_story_list.html` fragment for requests carrying `HX-Request: true`.
- `GET /health` returns HTTP 200 while the web process is running. It is a
  liveness check and does not reflect individual feed failures.
- HTMX is self-hosted at `/static/htmx.min.js`; styles are self-hosted at
  `/static/style.css`. Links and GET forms remain usable without JavaScript.
- SQLite persistence and scheduled refresh are not wired into the current
  binary yet. See [deployment and operations](operations/deployment.md) for
  container deployment and the backup limitation.

## Initial data model
```text
articles(id, url UNIQUE, url_hash, title, summary, source, published_at, fetched_at)
article_categories(article_id, category)       -- developer | ai | cybercrime
feed_state(source, etag, last_modified, last_fetch_at, last_error)
```

Store article metadata only: title, short summary, link, source, and publication time. Never store full article text.

## Planned crates
`tokio`, `reqwest`, `feed-rs`, `axum`, `askama`, `sqlx` (sqlite), `serde` + `toml`, `thiserror`, `anyhow` (only in `main`), `tracing`, `chrono`, `url`.
Add another crate only when there is a clear reason.

## Key design decisions
- Keep pipeline functions pure: accept data and return data without I/O. Test them without mocks.
- Use the `FeedSource` trait so a new non-RSS source can be added with a new implementation.
- Isolate errors per source: return a `Result` for each source, log failures, and keep the loop running.
- Keep category keywords in configuration so they can change without recompiling.
- Category keyword rules live in `config/categories.toml`; `tests/fixtures/classification_cases.toml` preserves reviewed live-feed headlines and expected labels.
- For the web interface, use [Impeccable](https://impeccable.style/docs/) when shaping and reviewing the UI. Establish product and visual context before implementation, and design around scanning and filtering news instead of falling back to generic dashboard patterns.
- Phase 2 (optional): add an LLM classifier behind the same `Classifier` trait.

## Known constraints and risks
- Reuters has no verified public technology RSS endpoint, and the AP RSS endpoint found during source verification requires client credentials. Keep both unconfigured until an authorized public feed or API is available.
- CNN's documented Technology RSS feed is stale; the latest item found was from August 2024. Keep it unconfigured until a fresh feed is available.
- BBC Indonesia RSS documentation points to an archived feed service; the initial source configuration uses the current English-language BBC Technology feed.
- Keyword classification can be wrong (for example, "Apple" may refer to a fruit or a company). Maintain the keyword list and test ambiguous cases.
- Feed URLs can change. The current one-shot fetch logs a source error and continues; the scheduler should persist `last_error` when added.

## Related decision
- [ADR 0001: SQLite and server-rendered HTML for v1](0001-sqlite-ssr.md).
