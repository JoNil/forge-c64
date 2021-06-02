CLANG=./llvm-mos/build/bin/clang --config llvm-mos-sdk/build/commodore/64.cfg -Os

all: src/forge.prg src/forge.s 

src/forge.prg: src/lib.ll Makefile
	${CLANG} src/lib.ll -o src/forge.prg

src/forge.s: src/lib.ll Makefile
	${CLANG} src/lib.ll -o src/forge.s -Wl,--lto-emit-asm

src/lib.ll: src/lib.rs Makefile
	rustc +nightly src/lib.rs --emit=llvm-ir --crate-type=rlib -C debuginfo=0 -C opt-level=s -o src/lib.ll

clean:
	rm -f src/*.ll
	rm -f src/*.s
	rm -f src/*.prg