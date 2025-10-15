#!/bin/bash

set -e

source ./toolkit.sh

export MSYS2_ARG_CONV_EXCL="*"

docker run -it \
    --mount type=bind,src="$(pwd)",target=/home/mos/forge-c64 \
    --mount type=bind,src="$(pwd)/target/mos-c64-none/release",target=/home/mos/forge-c64/out \
    --mount type=volume,source=forge_cargo_git,target=/home/mos/.cargo/git \
    --mount type=volume,source=forge_cargo_registry,target=/home/mos/.cargo/registry \
    --mount type=volume,source=forge_target,target=/home/mos/forge-c64/target \
    $DOCKER_TAG