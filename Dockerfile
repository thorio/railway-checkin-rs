FROM rust:1.70-bullseye AS builder
RUN rustup target add x86_64-unknown-linux-musl
RUN apt-get update && export DEBIAN_FRONTEND=noninteractive \
	&& apt-get -y install --no-install-recommends musl-tools

# precompile dependencies (so docker can cache them)
RUN cargo new src
WORKDIR /src
COPY Cargo.toml Cargo.lock .
RUN cargo build --locked --target x86_64-unknown-linux-musl --release
RUN rm -rf src rm target/x86_64-unknown-linux-musl/release/deps/railway_checkin*

# compile actual binary
COPY src src
RUN cargo install --locked --target x86_64-unknown-linux-musl --path .

# create final image
FROM scratch
COPY --from=builder /usr/local/cargo/bin/railway-checkin-rs /binary
CMD ["/binary"]
