#!/bin/bash

set -e

source ./toolkit.sh

docker run \
    -it \
    --mount src="$(pwd)",target=/home/mos/forge-c64,type=bind \
    $DOCKER_TAG