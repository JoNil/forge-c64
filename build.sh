#!/bin/bash
docker run -it --mount src="$(pwd)",target=/home/mos/forge-c64,type=bind mrkits/rust-mos bash -c "cd forge-c64 && cargo build --release"
docker container prune -f
