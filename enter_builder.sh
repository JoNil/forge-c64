#!/bin/bash

set -e

docker run -it --mount src="$(pwd)",target=/home/mos/forge-c64,type=bind mrkits/rust-mos bash