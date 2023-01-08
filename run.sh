#!/bin/bash

set -e

./build.sh
./tools/ef3utils/ef3usb.exe com4 s target/mos-c64-none/release/forge-c64
