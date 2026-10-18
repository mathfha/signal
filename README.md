# Signal

Signal brings technology stories from English RSS feeds into one
filterable page. It labels stories as Developer, AI, or Cybercrime using
keyword rules and shows each story's title, summary, source, publication time,
and link to the original article.

Signal fetches feeds when it starts and keeps the results in memory. A failed
feed does not stop the others. The current version does not yet save stories
to SQLite or refresh feeds on a schedule.

## Project status

Signal is still in development. Scheduled feed refresh, story normalization,
duplicate detection, and persistent storage are not implemented yet. The
[product requirements](docs/prd.md) and
[implementation plan](docs/implementation-plan.md) track the remaining work.

## Run locally

Install Rust, then start the app from the project directory:

```sh
RUST_LOG=info cargo run
```

Open <http://127.0.0.1:3000>. Set `TECHPULSE_ADDR` to change the bind address.
Use `RUST_LOG=debug cargo run` to see detailed logs.

Feed URLs are configured in `config/sources.toml`. Category keywords are in
`config/categories.toml`. Signal currently reads feeds from BBC News, Al
Jazeera, The Guardian, The New York Times, and DW News.

## Run with Docker

Build and start the service with Docker Compose:

```sh
docker compose up --build --detach
```

The app listens on port 3000 inside the container. Docker checks
`GET /health` to confirm the web process is responding. See the
[deployment guide](docs/operations/deployment.md) for setup and runtime limits.

## Checks

```sh
cargo fmt --check
cargo clippy --all-targets -- -D warnings
cargo test
```

## Project docs

| Topic | File |
|---|---|
| Product requirements | [docs/prd.md](docs/prd.md) |
| Architecture | [docs/overview.md](docs/overview.md) |
| Technical decision | [docs/0001-sqlite-ssr.md](docs/0001-sqlite-ssr.md) |
| Clean code guidelines | [docs/clean-code.md](docs/clean-code.md) |
| Implementation plan | [docs/implementation-plan.md](docs/implementation-plan.md) |
| Deployment and operations | [docs/operations/deployment.md](docs/operations/deployment.md) |
