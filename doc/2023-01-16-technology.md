### We need to

- Load an ELF from disk
- Lift the machine code of BBs to an IR (do you want to implement an x86 disassembler? me neither)
- Lots of algorithmic stuff, some persistent data structures
- Handle syscalls
- Visualize `MemDag` and `BbCfg`.

### Related

- ELF/(other formats) loading is commoditized. https://crates.io/crates/object
- Valgrind lifting and IR. https://sourceware.org/git/?p=valgrind.git;a=tree;f=VEX/pub;hb=HEAD
- Some LLVM decompiler thing. https://github.com/avast/retdec
- Machine code to LLVM translator. https://github.com/lifting-bits/remill
- Lifter to LLVM, using Remill. https://github.com/lifting-bits/mcsema
- Lifter to LLVM. https://github.com/aengelke/rellume
- https://github.com/angr/angr
- https://github.com/BinaryAnalysisPlatform/bap
- https://alastairreid.github.io/RelatedWork/notes/binary-analysis/
- Graphviz? GUI toolkit?

### Scope

- Support subset of x86, x86_64, aarch32, aarch64, mips32, mips64, riscv64
- Support subset of windows, linux (PE/ELF but particularly different syscalls)
- What syscalls to implement (linux has too many)
- Do we want to build anything graphical? Or just a library and some demo scripts?
