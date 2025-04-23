# -- Stage 1: Builder -- #
FROM rust:slim AS builder

WORKDIR /usr/src/senra

RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

COPY . .

RUN cargo build --release --package senra_server
RUN cargo install sqlx-cli
RUN sqlx database create --database-url sqlite:file:shaderlab.db

# -- Stage 2: Runtime -- #
FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /opt/senra

RUN mkdir -p /var/lib/senra && chmod 777 /var/lib/senra

COPY --from=builder /usr/src/senra/target/release/senra_server /usr/local/bin/
COPY --from=builder /usr/src/senra/shaderlab.db /var/lib/senra/shaderlab.db

ENV HOST=0.0.0.0
ENV PORT=3000
ENV DATABASE_URL=sqlite:file:/var/lib/senra/shaderlab.db

EXPOSE $PORT

CMD ["senra_server"]
