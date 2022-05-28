#!/bin/bash
docker run -it --mount src=`pwd`,target=/home/mos/forge-c64,type=bind mrkits/rust-mos cargo build --release -Zbuild-std=core --manifest-path=forge-c64/Cargo.toml --target=rust-mos-target/mos-c64-none.json
docker container prune -f