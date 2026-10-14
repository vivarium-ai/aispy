#!/bin/sh
set -eu

sudo apt update
sudo apt upgrade -y

sudo apt install -y \
  make \
  curl \
  build-essential \
  clang

curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
