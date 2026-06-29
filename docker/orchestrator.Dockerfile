FROM rust:1-bookworm AS build

WORKDIR /app/orchestrator
ENV CARGO_TARGET_DIR=/tmp/cargo-target
COPY orchestrator/ ./
RUN cargo build --release

FROM debian:bookworm-slim

RUN apt-get update \
    && apt-get install -y --no-install-recommends ca-certificates \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app/orchestrator
COPY --from=build /tmp/cargo-target/release/orchestrator /usr/local/bin/orchestrator
COPY orchestrator/src/prompt ./src/prompt

EXPOSE 3000
CMD ["/usr/local/bin/orchestrator"]
