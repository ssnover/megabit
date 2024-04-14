#!/bin/bash

set -ex

BOSSAC_PATH="$HOME/.arduino15/packages/arduino/tools/bossac/1.9.1-arduino2"
BIN_DIR="${CARGO_TARGET_DIR:-"$(pwd)/target"}/thumbv7em-none-eabi/release"
FW_NAME="megabit-coproc-nano-ble-rgb"

arm-none-eabi-objcopy --output-target=binary $BIN_DIR/$FW_NAME $BIN_DIR/$FW_NAME.bin

$BOSSAC_PATH/bossac -d -p /dev/ttyACM0 -U -i -e -w -R $BIN_DIR/$FW_NAME.bin
