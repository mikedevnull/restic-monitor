# restic-monitor

A Rust application with a reusable library crate and a command-line binary.

## Build and test

```sh
cargo fmt --all -- --check
cargo test --all-targets --all-features
cargo build --release
```

Run the binary with `cargo run`.

## Docker

Build and run the container image locally:

```sh
docker build -t restic-monitor .
docker run --rm restic-monitor
```

The GitHub Actions CI workflow checks formatting, runs Clippy and tests, and builds the release binary. The separate Docker image workflow builds the container image without publishing it.