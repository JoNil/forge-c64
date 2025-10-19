#!/bin/bash

set -e

source ./toolkit.sh

export MSYS2_ARG_CONV_EXCL="*"

rm target/mos-c64-none/release/*.s || true
rm target/mos-c64-none/release/forge-c64 || true

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

        sudo chown mos:mos /home/mos/.cargo/git
        sudo chown mos:mos /home/mos/.cargo/registry
        sudo chown mos:mos /home/mos/forge-c64/target

        cd forge-c64
        rm target/mos-c64-none/release/deps/*.s || true
        cargo build --release
        cargo rustc --release -- --emit asm
        cp target/mos-c64-none/release/forge-c64 out/
        cp target/mos-c64-none/release/deps/*.s out/'
docker container prune -f > /dev/null
du -bh target/mos-c64-none/release/forge-c64
