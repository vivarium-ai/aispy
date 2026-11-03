#!/bin/sh
set -eu

sudo apt update
sudo apt upgrade -y

sudo apt install -y \
  make \
  curl \
  build-essential \
  clang \
  llvm \
  libclang-dev \
  bpftool \
  podman

curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y

rustup component add rust-src --toolchain nightly

cargo install bindgen-cli
cargo install --git https://github.com/aya-rs/aya -- aya-tool

curl https://install.duckdb.org | bash
echo PATH="$HOME/.duckdb/cli/latest":"$PATH" >> ~/.bashrc
