#!/bin/bash

set -ex

cargo build --release --bin megabit-monocolor-coproc --features="dot_matrix"
cargo build --release --bin megabit-rgb-coproc --features="rgb_matrix"