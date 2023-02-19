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

FROM scratch as check
COPY --from=dep_check /app/recipe.json recipe-dep-check.json
COPY --from=test /app/recipe.json recipe-test.json

FROM builder as release
WORKDIR /app/gadget-worker

RUN cargo install -q worker-build && worker-build --release
RUN yarn run wrangler publish
