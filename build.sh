#!/usr/bin/bash

trunk build --features "frontend" --no-default-features simulator/index.html
cargo build --bin megabit-coproc-simulator --features "backend"
cargo build --bin megabit-runner