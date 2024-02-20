#!/usr/bin/bash

set -ex

cd simulator/frontend
trunk build --release
cd -
cargo build --bin megabit-coproc-simulator
cargo build --bin megabit-runner
cd example-apps/game-of-life
cargo build --release
cd -
cd example-apps/hello-world
cargo build --release
cd -
cd example-apps/nyan-cat
cargo build --release
cd -
cd coproc-embassy
cargo build --release
cd -