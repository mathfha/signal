# TechPulse

TechPulse is a Rust application that reads English-language international news
feeds. It fetches feeds from BBC News, Al Jazeera, The Guardian, The New York
Times, and DW, then serves a filterable news index. A failure from one source
does not prevent other feeds from being processed.

The web interface uses Askama for server-rendered HTML and self-hosted HTMX for
partial page interactions. See [the architecture decision](docs/0001-sqlite-ssr.md).

## Run

Install Rust, then run the application from the project directory:

```sh
RUST_LOG=info cargo run
```

Open <http://127.0.0.1:3000>. The app fetches configured feeds at startup.
Override the bind address with `TECHPULSE_ADDR`.

Feed URLs are listed in `config/sources.toml`; category matching terms are in
`config/categories.toml`. Reuters and AP remain unconfigured
until an authorized public feed is verified. CNN's documented feeds are stale;
the latest Technology feed item found was from August 2024. The BBC Indonesia
RSS documentation points to an archived service, so this setup uses the
English-language BBC feed. To see more detailed logs, use `RUST_LOG=debug cargo
run`.

Run tests and required checks with:

```sh
cargo test
cargo fmt --check
cargo clippy --all-targets -- -D warnings
```

## Documentation index

| Topic | File | Status |
|---|---|---|
| Product requirements | [prd.md](docs/prd.md) | Draft |
| Architecture | [overview.md](docs/overview.md) | Draft |
| Technical decision (ADR) | [0001-sqlite-ssr.md](docs/0001-sqlite-ssr.md) | Accepted |
| Clean code | [clean-code.md](docs/clean-code.md) | Draft |
| Implementation plan | [implementation-plan.md](docs/implementation-plan.md) | Draft |
| Deployment and operations | [deployment.md](docs/operations/deployment.md) | Current runtime limits documented |

## Container deployment

Build and start the app with `docker compose up --build --detach`. The
container health check uses `GET /health`. See the
[deployment guide](docs/operations/deployment.md) for operational details and
the current SQLite backup limitation.
