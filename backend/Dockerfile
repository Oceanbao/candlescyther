FROM rust:1.90.0 AS chef

RUN cargo install sccache --version ^0.10
RUN cargo install cargo-chef --version ^0.1
ENV RUSTC_WRAPPER=sccache \
  SCCACHE_DIR=/sccache

FROM chef AS planner

WORKDIR /app
COPY . .
RUN --mount=type=cache,target=/usr/local/cargo/registry \
  --mount=type=cache,target=$SCCACHE_DIR,sharing=locked \
  cargo chef prepare --recipe-path recipe.json

FROM chef as builder

ENV SQLX_OFFLINE=true

WORKDIR /app

COPY --from=planner /app/recipe.json recipe.json
RUN --mount=type=cache,target=/usr/local/cargo/registry \
  --mount=type=cache,target=$SCCACHE_DIR,sharing=locked \
  cargo chef cook --release --recipe-path recipe.json
COPY . .
RUN --mount=type=cache,target=/usr/local/cargo/registry \
  --mount=type=cache,target=$SCCACHE_DIR,sharing=locked \
  --mount=type=cache,target=/app/target \
  cargo build --release --bin server && \
  mv ./target/release/server server

# Runtime image
FROM gcr.io/distroless/cc-debian12

# Run as "app" user
# RUN groupadd -r appuser && useradd --no-log-init -r -g appuser appuser
# RUN groupadd --system appuser && useradd --system -g appuser appuser
# RUN useradd -ms /bin/bash app
# USER appuser

WORKDIR /app

# Get compiled binaries from builder's cargo install directory
COPY --from=builder /app/server server
COPY --from=builder /app/migrations migrations

EXPOSE 8080

ENV RUST_LOG=debug

# Run the app
CMD ["./server"]
