FROM rust:1-bookworm AS builder

WORKDIR /build
COPY Cargo.toml Cargo.lock ./
COPY src ./src
COPY templates ./templates
COPY static ./static
COPY config ./config
RUN cargo build --release --locked

FROM debian:bookworm-slim AS runtime

RUN apt-get update \
    && apt-get install --yes --no-install-recommends ca-certificates curl \
    && rm -rf /var/lib/apt/lists/* \
    && useradd --system --uid 10001 --no-create-home --shell /usr/sbin/nologin techpulse

WORKDIR /app
COPY --from=builder /build/target/release/techpulse /usr/local/bin/techpulse
COPY --from=builder /build/config ./config

ENV TECHPULSE_ADDR=0.0.0.0:3000
EXPOSE 3000
USER 10001:10001

HEALTHCHECK --interval=30s --timeout=5s --start-period=60s --retries=3 \
    CMD curl --fail --silent http://127.0.0.1:3000/health || exit 1

ENTRYPOINT ["/usr/local/bin/techpulse"]
