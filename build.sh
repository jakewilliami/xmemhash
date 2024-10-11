#!/usr/bin/env sh

set -xe
trap 'exit 1' INT

MODE="release"  # previously: "debug"
cargo build --$MODE
strip ./target/$MODE/$(basename "$PWD")
cp -f ./target/$MODE/$(basename "$PWD") ./
