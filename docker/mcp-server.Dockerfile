FROM rust:1-bookworm AS build

ARG PACKAGE
WORKDIR /app/mcp_server
COPY mcp_server/ ./
RUN cargo build --release -p ${PACKAGE}

FROM debian:bookworm-slim

ARG PACKAGE
RUN apt-get update \
    && apt-get install -y --no-install-recommends ca-certificates \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app/mcp_server
COPY --from=build /app/mcp_server/target/release/${PACKAGE} /usr/local/bin/${PACKAGE}

EXPOSE 3101 3102 3300 3400
CMD ["/bin/sh", "-c", "exec /usr/local/bin/${PACKAGE}"]
