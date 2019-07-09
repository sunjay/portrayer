#!/usr/bin/env bash

set -e

for file in $(ls examples/*.rs); do
  echo "============================================="
  echo
  echo "Running example: $(basename ${file%.*})"
  time RUST_BACKTRACE=1 cargo run --release --example "$(basename ${file%.*})" "$@"
  echo
  echo "============================================="
done
