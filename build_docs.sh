#!/bin/bash

set -e

source ./toolkit.sh

docker run -it \
    --mount src="$(pwd)",target=/home/mos/forge-c64,type=bind \
    --mount src="$(pwd)/.cargo/git",target=/home/mos/.cargo/git,type=bind \
    --mount src="$(pwd)/.cargo/registry",target=/home/mos/.cargo/registry,type=bind \
    $DOCKER_TAG \
    bash -c "source .cargo/env && cd forge-c64 && cargo doc"
docker container prune -f > /dev/null
