FROM rustlang/rust:nightly as builder

# create a new empty shell project
RUN USER=root cargo new --bin gadget-server
WORKDIR /gadget-server

# copy over your manifests
COPY ./Cargo.lock ./Cargo.lock
COPY ./Cargo.toml ./Cargo.toml

RUN mkdir src/gadget && touch src/gadget/lib.rs

# this build step will cache your dependencies
RUN cargo build --release
RUN rm -rf src/

# copy your source tree
COPY ./src ./src

# build for release
RUN find  target/release/ -type f -name '*gadget*' -delete
RUN cargo build --release
RUN cargo test

# our final base
FROM rust:1.23

# copy the build artifact from the build stage
COPY --from=builder /gadget-server/target/release/gadget-server .

ENV TINI_VERSION v0.18.0
ADD https://github.com/krallin/tini/releases/download/${TINI_VERSION}/tini /tini
RUN chmod +x /tini

ENTRYPOINT ["/tini", "--"]
# set the startup command to run your binary
CMD [ "./gadget-server"]