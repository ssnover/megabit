#!/bin/bash

BOSSAC_PATH="$HOME/.arduino15/packages/arduino/tools/bossac/1.9.1-arduino2"
BIN_DIR="${CARGO_TARGET_DIR:-"$(pwd)/target"}/thumbv7em-none-eabi/release"

arm-none-eabi-objcopy --output-target=binary $BIN_DIR/megabit-coproc $BIN_DIR/megabit-coproc.bin

$BOSSAC_PATH/bossac -d -p /dev/ttyACM0 -U -i -e -w -R $BIN_DIR/megabit-coproc.bin
