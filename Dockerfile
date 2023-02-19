# syntax=docker/dockerfile:1.4
FROM rust:bullseye as chef
COPY rust-toolchain.toml rust-toolchain.toml
RUN <<EOT
#!/usr/bin/env bash
set -euxo pipefail

apt-get update
apt-get install protobuf-compiler -y
cargo install cargo-chef
EOT

WORKDIR /app

FROM chef AS planner
COPY . .
RUN cargo chef prepare  --recipe-path recipe.json

FROM chef AS builder
COPY --from=planner /app/recipe.json recipe.json
# Build dependencies - this is the caching Docker layer!
RUN cargo chef cook --release --recipe-path recipe.json
COPY . .

FROM builder as dep_check
RUN rustup toolchain install nightly --allow-downgrade --profile minimal

RUN <<EOT
#!/usr/bin/env bash
set -euxo pipefail

cargo +nightly build --release
cargo +nightly install cargo-udeps --locked
cargo +nightly udeps --release
EOT

FROM builder as test
RUN <<EOT
#!/usr/bin/env bash
set -euxo pipefail

rustup component add rustfmt clippy

cargo test --release
cargo fmt --check
cargo clippy --release
EOT

FROM scratch as check
COPY --from=dep_check /app/recipe.json recipe-dep-check.json
COPY --from=test /app/recipe.json recipe-test.json

FROM builder as release
RUN cargo build --release --bin gadget-server
RUN /app/target/release/gadget-server --help

FROM debian:bullseye-slim AS runtime
RUN apt-get update && apt-get install tini -y
WORKDIR /app
COPY --from=release /app/target/release/gadget-server /usr/local/bin
ENTRYPOINT ["/usr/bin/tini", "--"]
CMD ["/usr/local/bin/gadget-server"]