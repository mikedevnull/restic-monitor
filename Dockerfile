FROM rust:1.85-slim AS builder
WORKDIR /app
COPY Cargo.toml ./
COPY src ./src
RUN cargo build --release

FROM debian:bookworm-slim
RUN useradd --system --no-create-home --uid 10001 app
COPY --from=builder /app/target/release/restic-monitor /usr/local/bin/restic-monitor
USER app
ENTRYPOINT ["restic-monitor"]