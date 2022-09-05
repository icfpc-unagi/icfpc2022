#!/bin/bash -eu

if (($# > 0)); then
  nice cargo run --release --bin optimize -- --allow-not-best --submission-ids "$*"
else
  # nice cargo run --release --bin optimize
  nice cargo run --release --bin optimize -- --allow-not-best --latest 100 --program-name-not optimize
  # nice cargo run --release --bin optimize -- -a --program-name chokudai
fi
