# C64 Experiments

## Build
git clone https://github.com/JoNil/forge-c64

cd forge-c64
git submoduel init
git submoduel update

cd llvm-mos
cmake -C clang/cmake/caches/MOS.cmake -G "Ninja" -S llvm -B build \
   -DLLVM_INSTALL_TOOLCHAIN_ONLY=OFF \
   -DLLVM_BUILD_LLVM_DYLIB=ON -DLLVM_LINK_LLVM_DYLIB=ON \
   -DLLVM_INSTALL_UTILS=ON -DLLVM_BUILD_UTILS=ON -DLLVM_TOOLCHAIN_UTILITIES=FileCheck \
   -DLLVM_TOOLCHAIN_TOOLS="llvm-addr2line;llvm-ar;llvm-cxxfilt;llvm-dwarfdump;llvm-mc;llvm-nm;llvm-objcopy;llvm-objdump;llvm-ranlib;llvm-readelf;llvm-readobj;llvm-size;llvm-strings;llvm-strip;llvm-symbolizer;llvm-config;llc" \
   -DLIBXML2_LIBRARY=/usr/lib/x86_64-linux-gnu/libxml2.so \
   -DLLVM_TARGETS_TO_BUILD="MOS;X86" \
   -DLLVM_ENABLE_PROJECTS="clang;lld;lldb"
cmake --build build -t install


## Links
- https://github.com/llvm-mos/llvm-mos
- https://github.com/mrk-its/rust-mos
- http://forum.6502.org/viewtopic.php?p=84048

## C64 Links
- https://nybblesandbytes.net/6502
- https://www.youtube.com/watch?v=kxc46GNVDIk
- https://www.c64-wiki.com/wiki/SID
- https://www.pagetable.com/c64ref/kernal/
- https://www.pagetable.com/c64ref/c64mem/
- https://www.c64-wiki.com/wiki/Memory_Map
- https://dustlayer.com/tutorials
- http://www.0xc64.com/2017/02/12/tutorial-4x4-dynamic-text-scroller/
- http://1amstudios.com/2014/12/07/c64-smooth-scrolling/
- http://www.zimmers.net/cbmpics/cbm/c64/c64prg.txt
- https://www.commodore.ca/manuals/c64_programmers_reference/c64-programmers_reference_guide-03-programming_graphics.pdf
- http://sta.c64.org/cbm64mem.html
- https://en.wikipedia.org/wiki/MOS_Technology_VIC-II
- https://github.com/demesos/LAMAlib
- https://github.com/demesos/LAMAlib/blob/master/LAMAlib-structured.inc
- http://mynesdev.blogspot.com/2013/09/getting-most-out-of-ca65-part-1.html