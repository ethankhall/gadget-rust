FROM rustlang/rust:nightly as builder

# create a new empty shell project
RUN USER=root cargo new --bin gadget-server
WORKDIR /gadget-server

# copy over your manifests
COPY ./Cargo.lock ./Cargo.lock
COPY ./Cargo.toml ./Cargo.toml

# this build step will cache your dependencies
RUN cargo build --release
RUN rm src/*.rs

# copy your source tree
COPY ./src ./src

# build for release
RUN rm ./target/release/deps/gadget_server*
RUN cargo build --release

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