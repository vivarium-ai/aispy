# syntax=docker/dockerfile:1

FROM nvidia/cuda:13.4.1-devel-ubuntu24.04 AS builder

ARG DEBIAN_FRONTEND=noninteractive

RUN apt-get update \
    && apt-get install -y --no-install-recommends \
        build-essential \
        ca-certificates \
        clang \
        curl \
        git \
        libssl-dev \
        make \
        pkg-config \
    && rm -rf /var/lib/apt/lists/*

RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs \
    | sh -s -- -y --profile minimal --default-toolchain stable

ENV PATH="/root/.cargo/bin:${PATH}"

WORKDIR /workspaces/aispy

COPY . .

RUN cargo check --lib --no-default-features \
    && cargo check --lib --no-default-features --features cuda \
    && cargo build --release --bin aispy --features cli


FROM nvidia/cuda:13.4.1-runtime-ubuntu24.04 AS runtime

ARG DEBIAN_FRONTEND=noninteractive

RUN apt-get update \
    && apt-get install -y --no-install-recommends \
        ca-certificates \
    && rm -rf /var/lib/apt/lists/*

COPY --from=builder \
    /workspaces/aispy/target/release/aispy \
    /usr/local/bin/aispy

ENTRYPOINT ["/usr/local/bin/aispy"]
