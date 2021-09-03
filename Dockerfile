FROM rust:buster as rust-builder

WORKDIR /gadget

# copy over your manifests
COPY ./Cargo.lock ./Cargo.lock
COPY ./Cargo.toml ./Cargo.toml

RUN USER=root cargo new --bin gadget-server
COPY gadget-server/Cargo.toml /gadget/gadget-server/Cargo.toml
WORKDIR /gadget/gadget-server

RUN cargo build --release
RUN rm src/*.rs
RUN rm /gadget/target/release/deps/gadget*

WORKDIR /gadget
ADD gadget-server /gadget/gadget-server

# this build step will cache your dependencies
RUN cargo install --path ./gadget-server
RUN /usr/local/cargo/bin/gadget --help

FROM node:13.5-stretch as node-builder

# Image to build website in
WORKDIR /gadget

COPY gadget-ui /gadget/gadget-ui
WORKDIR /gadget/gadget-ui

RUN yarn
RUN yarn build

# verify linked deps
FROM debian:buster-slim

RUN apt-get update && apt-get install -y libpq5 && apt-get clean 

# copy the build artifact from the build stage
COPY --from=rust-builder /usr/local/cargo/bin/gadget /app/bin/gadget
RUN /app/bin/gadget --help

# our final base
FROM debian:buster-slim

RUN apt-get update && apt-get install -y libpq5 && apt-get clean 

# copy the build artifact from the build stage
COPY --from=rust-builder /usr/local/cargo/bin/gadget /app/bin/gadget
COPY --from=node-builder /gadget/gadget-ui/dist /app/public
COPY gadget-server/migrations /app/migrations

ENV UI_PATH /app/public
ENV TINI_VERSION v0.18.0
ADD https://github.com/krallin/tini/releases/download/${TINI_VERSION}/tini /tini
RUN chmod +x /tini

WORKDIR /app

ENTRYPOINT ["/tini", "--"]
# set the startup command to run your binary
CMD [ "/app/bin/gadget"]

