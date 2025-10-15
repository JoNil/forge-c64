#!/bin/bash

set -e

source ./toolkit.sh

export MSYS_NO_PATHCONV=1
export MSYS2_ARG_CONV_EXCL="*"

docker run -it \
    --mount type=bind,src="$(pwd)",target=/home/mos/forge-c64 \
    --mount type=bind,src="$(pwd)/target/mos-c64-none/release",target=/home/mos/forge-c64/out \
    --mount type=volume,source=forge_cargo_git,target=/home/mos/.cargo/git \
    --mount type=volume,source=forge_cargo_registry,target=/home/mos/.cargo/registry \
    --mount type=volume,source=forge_target,target=/home/mos/forge-c64/target \
    $DOCKER_TAG \
    bash -lc '
        set -e
        source .cargo/env
        mkdir -p /home/mos/.cargo/git /home/mos/.cargo/registry /home/mos/forge-c64/target
        sudo chown -R mos:mos /home/mos/.cargo /home/mos/forge-c64/target
        cd forge-c64
        cargo build --release
        cp target/mos-c64-none/release/forge-c64 out/'
docker container prune -f > /dev/null
du -bh target/mos-c64-none/release/forge-c64
