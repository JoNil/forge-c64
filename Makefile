CLANG=./llvm-mos/build/bin/clang --config llvm-mos-sdk/build/commodore/64.cfg -O0

all: src/forge.prg src/forge.s

src/forge.prg: src/lib.ll src/main.c
	${CLANG} src/main.c src/lib.ll -o src/forge.prg

src/forge.s: src/lib.ll src/main.c
	${CLANG} src/main.c src/lib.ll -o src/forge.s -Wl,--lto-emit-asm

src/lib.ll: src/lib.rs
	rustc src/lib.rs --emit=llvm-ir --crate-type=rlib -C debuginfo=0 -C opt-level=1 -o src/lib.ll
   