# -- Stage 1: Builder -- #
FROM rust:slim AS builder

WORKDIR /usr/src/senra

RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

COPY . .

RUN cargo build --release --package senra_server

# -- Stage 2: Runtime -- #
FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /opt/senra

COPY --from=builder /usr/src/senra/target/release/senra_server /usr/local/bin/

ENV HOST=0.0.0.0
ENV PORT=8080

EXPOSE $PORT

CMD ["senra_server"]
