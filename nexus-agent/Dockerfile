# ══════════════════════════════════════════════════════════
# Nexus Agent — 多阶段 Docker 构建
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

COPY --from=builder /app/target/release/nexus-agent /usr/local/bin/nexus-agent

# 数据目录（密钥、序列号）
RUN mkdir -p /data
VOLUME /data

# 环境变量默认值
ENV WEBHOOK_PORT=8443
ENV DATA_DIR=/data
ENV RUST_LOG=nexus_agent=info

EXPOSE 8443

HEALTHCHECK --interval=30s --timeout=5s --start-period=10s --retries=3 \
    CMD curl -f http://localhost:8443/health || exit 1

ENTRYPOINT ["nexus-agent"]
