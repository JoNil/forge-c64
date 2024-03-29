#!/bin/bash

set -e

docker run -it --mount src="$(pwd)",target=/home/mos/forge-c64,type=bind --mount src="$(pwd)/.cargo/git",target=/home/mos/.cargo/git,type=bind --mount src="$(pwd)/.cargo/registry",target=/home/mos/.cargo/registry,type=bind mrkits/rust-mos bash -c "cd forge-c64 && cargo rustc --release -- --emit asm"
docker container prune -f > /dev/null
du -bh target/mos-c64-none/release/forge-c64