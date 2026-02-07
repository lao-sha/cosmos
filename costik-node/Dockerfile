# ══════════════════════════════════════════════════════════
# Costik Node — 多阶段 Docker 构建
# ══════════════════════════════════════════════════════════

# Stage 1: 编译
FROM rust:1.82-slim AS builder

RUN apt-get update && apt-get install -y pkg-config libssl-dev && rm -rf /var/lib/apt/lists/*

WORKDIR /app
COPY Cargo.toml Cargo.lock* ./
COPY src/ ./src/

RUN cargo build --release

# Stage 2: 运行时
FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/target/release/costik-node /usr/local/bin/costik-node

RUN mkdir -p /data
VOLUME /data

ENV LISTEN_PORT=8080
ENV GOSSIP_PORT=9090
ENV DATA_DIR=/data
ENV RUST_LOG=costik_node=info

EXPOSE 8080 9090

HEALTHCHECK --interval=30s --timeout=5s --start-period=10s --retries=3 \
    CMD curl -f http://localhost:8080/health || exit 1

ENTRYPOINT ["costik-node"]
