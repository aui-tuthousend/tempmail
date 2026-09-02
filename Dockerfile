FROM rust:1.80-bookworm AS builder

WORKDIR /app

ARG PACKAGE

COPY Cargo.toml Cargo.lock ./
COPY crates ./crates

RUN cargo build --release -p "$PACKAGE" \
  && cp "target/release/$PACKAGE" /usr/local/bin/tempmail-service

FROM debian:bookworm-slim AS runtime

RUN apt-get update \
  && apt-get install -y --no-install-recommends ca-certificates \
  && rm -rf /var/lib/apt/lists/*

COPY --from=builder /usr/local/bin/tempmail-service /usr/local/bin/tempmail-service

ENTRYPOINT ["/usr/local/bin/tempmail-service"]
