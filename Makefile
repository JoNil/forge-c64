CLANG=llvm-mos/build/bin/clang --config llvm-mos-sdk/build/atari/800xl.cfg -O0

all: src/lib.xex src/lib.s

src/lib.xex: src/lib.ll src/main.c
   ${CLANG} src/main.c src/lib.ll -o src/lib.xex

src/lib.s: src/lib.ll src/main.c
   ${CLANG} src/main.c src/lib.ll -o src/lib.s -Wl,--lto-emit-asm

src/lib.ll: src/lib.rs
   rustc src/lib.rs --emit=llvm-ir --crate-type=rlib -C debuginfo=0 -C opt-level=1 -o src/lib.ll
   