# syntax=docker/dockerfile:1.4
####################################################################################################################################
#### COPIED FROM :: Alex Vincent :: https://github.com/f2calv/multi-arch-container-rust/blob/main/Dockerfile
####################################################################################################################################
FROM --platform=$BUILDPLATFORM rust AS base
WORKDIR /app
RUN apt-get update && apt-get upgrade -y
RUN rustup component add clippy
RUN rustup component add rustfmt

ARG TARGETPLATFORM
RUN \
if [ "$TARGETPLATFORM" = "linux/amd64" ]; then \
    apt-get install -y g++-x86-64-linux-gnu libc6-dev-amd64-cross libc6-dev; \
    rustup target add x86_64-unknown-linux-gnu ; \
    rustup toolchain install stable-x86_64-unknown-linux-gnu ; \
elif [ "$TARGETPLATFORM" = "linux/arm64" ]; then \
    apt-get install -y g++-aarch64-linux-gnu libc6-dev-arm64-cross libc6-dev; \
    rustup target add aarch64-unknown-linux-gnu ; \
    rustup toolchain install stable-aarch64-unknown-linux-gnu ; \
elif [ "$TARGETPLATFORM" = "linux/arm/v7" ]; then \
    apt-get install -y g++-arm-linux-gnueabihf libc6-dev-armhf-cross libc6-dev; \
    rustup target add armv7-unknown-linux-gnueabihf ; \
    rustup toolchain install stable-armv7-unknown-linux-gnueabihf ; \
fi
RUN apt install -y protobuf-compiler


FROM base AS dependencies
WORKDIR /app
#initialize an empty application & replace the dependencies file with our own (yes cargo chef can do this, but I feel this is simpler...)
RUN cargo init
COPY Cargo.toml Cargo.lock /app
ARG TARGETPLATFORM
RUN \
if [ "$TARGETPLATFORM" = "linux/amd64" ]; then \
    TARGET=x86_64-unknown-linux-gnu ; \
elif [ "$TARGETPLATFORM" = "linux/arm64" ]; then \
    TARGET=aarch64-unknown-linux-gnu ; \
    export CARGO_TARGET_AARCH64_UNKNOWN_LINUX_GNU_LINKER=aarch64-linux-gnu-gcc ; \
    export CC_aarch64_unknown_linux_gnu=aarch64-linux-gnu-gcc ; \
    export CXX_aarch64_unknown_linux_gnu=aarch64-linux-gnu-g++ ; \
elif [ "$TARGETPLATFORM" = "linux/arm/v7" ]; then \
    TARGET=armv7-unknown-linux-gnueabihf ; \
    export CARGO_TARGET_ARMV7_UNKNOWN_LINUX_GNUEABIHF_LINKER=arm-linux-gnueabihf-gcc ; \
    export CC_armv7_unknown_Linux_gnueabihf=arm-linux-gnueabihf-gcc ; \
    export CXX_armv7_unknown_linux_gnueabihf=arm-linux-gnueabihf-g++ ; \
fi \
&& cargo fetch --target $TARGET
#&& cargo build --release --target $TARGET
#https://github.com/f2calv/multi-arch-container-rust/issues/15



FROM dependencies AS source
COPY src src
COPY build.rs build.rs
COPY protos protos



FROM source AS build
ARG TARGETPLATFORM
RUN mkdir -p /build
RUN \
if [ "$TARGETPLATFORM" = "linux/amd64" ]; then \
    TARGET=x86_64-unknown-linux-gnu ; \
    echo 'TODO: need to complete and test building x86_64 FROM an arm platform??... ' ; \
elif [ "$TARGETPLATFORM" = "linux/arm64" ]; then \
    TARGET=aarch64-unknown-linux-gnu ; \
    export CARGO_TARGET_AARCH64_UNKNOWN_LINUX_GNU_LINKER=aarch64-linux-gnu-gcc ; \
    export CC_aarch64_unknown_linux_gnu=aarch64-linux-gnu-gcc ; \
    export CXX_aarch64_unknown_linux_gnu=aarch64-linux-gnu-g++ ; \
elif [ "$TARGETPLATFORM" = "linux/arm/v7" ]; then \
    TARGET=armv7-unknown-linux-gnueabihf ; \
    export CARGO_TARGET_ARMV7_UNKNOWN_LINUX_GNUEABIHF_LINKER=arm-linux-gnueabihf-gcc ; \
    export CC_armv7_unknown_Linux_gnueabihf=arm-linux-gnueabihf-gcc ; \
    export CXX_armv7_unknown_linux_gnueabihf=arm-linux-gnueabihf-g++ ; \
fi \
&& RUSTFLAGS="-C target-feature=+crt-static" cargo build --release --bin server --target $TARGET && mv /app/target/$TARGET/release /build/release


#FROM bitnami/minideb:latest
FROM gcr.io/distroless/cc AS final
LABEL org.opencontainers.image.source = "https://github.com/joostvdg/gitstafette-discovery"

COPY --from=build /build/release/server .

ENTRYPOINT ["./server"]
CMD ["-l", "0.0.0.0", "-p", "50051"]
## TODO - Verify ARM build
## TODO - Verify x86_64 Alpine build
## TODO - Verify ARM Alpine build
## TODO - looks like caching doesn't work because cross compilation