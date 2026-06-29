FROM rust:1-bookworm AS build

WORKDIR /app/resource_service
ENV CARGO_TARGET_DIR=/tmp/cargo-target
COPY resource_service/ ./
COPY resource_service_postgres.sql ../resource_service_postgres.sql
RUN cargo build --release --bin resource_service --bin resource_worker

FROM debian:bookworm-slim

RUN apt-get update \
    && apt-get install -y --no-install-recommends ca-certificates \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app/resource_service
COPY --from=build /tmp/cargo-target/release/resource_service /usr/local/bin/resource_service
COPY --from=build /tmp/cargo-target/release/resource_worker /usr/local/bin/resource_worker
COPY resource_service/config ./config

EXPOSE 3200
CMD ["/usr/local/bin/resource_service"]
