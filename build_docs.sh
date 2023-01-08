#!/bin/bash

set -e

docker run -it --mount src="$(pwd)",target=/home/mos/forge-c64,type=bind --mount src="$(pwd)/.cargo/git",target=/home/mos/.cargo/git,type=bind --mount src="$(pwd)/.cargo/registry",target=/home/mos/.cargo/registry,type=bind mrkits/rust-mos bash -c "cd forge-c64 && cargo doc"
docker container prune -f
