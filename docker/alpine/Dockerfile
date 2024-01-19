# syntax=docker/dockerfile:1
FROM rust AS base

# amd64-specific stage
ARG BIN_NAME="server"
FROM base as build-amd64
WORKDIR /usr/src/gitstafette-discovery
ENV TARGET=x86_64-unknown-linux-musl
RUN rustup target add x86_64-unknown-linux-musl
RUN apt update && apt install -y g++-mingw-w64-x86-64 protobuf-compiler musl-tools
RUN rustup target add x86_64-pc-windows-gnu
RUN rustup toolchain install stable-x86_64-pc-windows-gnu

COPY Cargo.toml Cargo.lock ./
COPY ./src ./src
COPY build.rs build.rs
COPY protos protos
RUN mkdir -p /build
RUN RUSTFLAGS="-C target-feature=+crt-static" cargo build --release --bin ${BIN_NAME} --target=$TARGET && mv /usr/src/gitstafette-discovery/target/$TARGET/release /build/release

# arm64-specific stage
ARG BIN_NAME="server"
FROM base as build-arm64
ARG APP_NAME="app"
ARG TARGET="aarch64-unknown-linux-musl"
RUN apt-get update
RUN apt-get install clang llvm protobuf-compiler musl-tools -y
RUN rustup target add $TARGET
RUN mkdir /usr/src/$APP_NAME
WORKDIR /usr/src/$APP_NAME

ENV CARGO_NET_GIT_FETCH_WITH_CLI=true

ENV CC_aarch64_unknown_linux_musl=clang
ENV AR_aarch64_unknown_linux_musl=llvm-ar
ENV CARGO_TARGET_AARCH64_UNKNOWN_LINUX_MUSL_RUSTFLAGS="-Clink-self-contained=yes -Clinker=rust-lld"
ENV CARGO_TARGET_AARCH64_UNKNOWN_LINUX_MUSL_RUNNER="qemu-aarch64 -L /usr/aarch64-linux-gnu"

COPY Cargo.toml Cargo.lock ./
COPY ./src ./src
COPY build.rs build.rs
COPY protos protos

RUN mkdir -p /build
RUN RUSTFLAGS="-C target-feature=+crt-static" cargo build --release --bin ${BIN_NAME} --target=$TARGET && mv /usr/src/$APP_NAME/target/$TARGET/release /build/release


# common steps
FROM build-${TARGETARCH} AS build
RUN echo "This stage is used on all architectures"
RUN ls -lath /build/release

# common steps
FROM alpine:3.19
ARG BIN_NAME="server"
LABEL org.opencontainers.image.source = "https://github.com/joostvdg/gitstafette-discovery"
LABEL org.opencontainers.image.description ="gitstafette-discovery is a tool to help you with the next version number of your project. It is based on the git history and the tags you have created. It is a simple tool that can be used in your CI/CD pipeline to determine the next version number of your project."
LABEL org.opencontainers.image.authors ="Joost van der Griendt <joostvdg@gmail.com>"
WORKDIR /work/
ENV RUST_LOG=info
ENV RUST_BACKTRACE=1

RUN addgroup -S appgroup && adduser -D -u 1000 gitstafette -G appgroup -h /work

COPY --from=build --chown=gitstafette:appgroup /build/release/${BIN_NAME} /work/discovery

USER gitstafette
RUN ls -lath /work

ENTRYPOINT ["./discovery"]
CMD ["-l", "0.0.0.0", "-p", "50051"]
