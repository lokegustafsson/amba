## Previous decisions

- If we use C bindings, we use Rust. The alternative is using the framework's language.

## Related projects

Support=general implies support for x86/x86_64/aarch32/aarch64/mips32/mips64

| Link                                                                        | License   | Extras?           | Lang       | IR           | Support                |
| --------------------------------------------------------------------------- | --------- | ----------------- | ---------- | ------------ | ---------------------- |
| https://sourceware.org/git/?p=valgrind.git;a=tree;f=VEX/pub;hb=HEAD         | GPLv2     |                   | Cabi       | VEX          | general                |
| https://github.com/avast/retdec                                             | MIT+other | -                 | ?          | -            | general                |
| https://github.com/lifting-bits/remill                                      | Apache2   | -                 | C++        | -            | x86/x86_64/aarch64     |
| https://github.com/lifting-bits/mcsema                                      | AGPLv3    | Built atop remill | ?          | -            | x86/x86_64             |
| https://github.com/aengelke/rellume                                         | LGPLv2    | -                 | ?          | LLVM         | x86_64/aarch64/riscv64 |
| https://github.com/angr/angr                                                | BSD       | High level        | Python     | -            |                        |
| https://github.com/BinaryAnalysisPlatform/bap                               | MIT       | -                 | Cabi/OCaml | -            | general                |
| https://web.archive.org/web/20110624024725/http://bitblaze.cs.berkeley.edu/ | Closed?   | ?                 | ?          | ?            | ?                      |
| https://github.com/radareorg/radare2                                        | LGPL      | -                 | Cabi/IPC   | ESIL (Forth) | general                |

## Notes

### RetDec

Quoted from https://github.com/avast/retdec/wiki/Capstone2LlvmIr

> Decompilation is one thing, but if someone would like to use RetDec framework for other purposes,
> he/she might want to have semantics for even very complex instructions. Right now, we would say
> that projects like QEMU or McSema are a better alternative in such a case. However, it might
> happen that someone will add complex semantics to Capstone2LlvmIr on their own - we currently have
> no such plans. This would not be easy, but if good groundworks were prepared, it might not be so
> bad. After all, someone had to hand-write these things in QEMU as well. Even if this happens, it
> would not be beneficial for decompilation (as already explained). So we would either have to keep
> these Capstone2LlvmIr translators separate, or have it all in one translator but be able to tell
> it what should and should not be translated - or which mode to use for which instruction.

So RetDec uses the capstone disassembler, then has a custom implementation for lifting to llvm ir.
This implementation is only approximately correct. As they say in the quote, it is good enough for
decompilation but not for emulation, look to QEMU or McSema. Not for us.

### McSema

Uses IDA Pro for binary disassembly and (static, not how we use the word) control flow graph
recovery. Not for us.

### remill

A large amout of C++ headers. Implementing bindings would be a pain. But it seems like a very
reasonable LLVM IR lifter for our use case.

### radare2

Used by a rust symbolic executor (over IPC) https://github.com/aemmitt-ns/radius

But does NOT SUPPORT many things, particularly floats and simd. I think?

## Reasonable options

Most options support {x86,arm}-{32,64}. Architecture support differences do not really matter for
us.

- Valgrind if we like VEX
- Remill if we like LLVM IR

# Extend QEMU with symbolic execution

## SymQEMU

https://github.com/eurecom-s3/symqemu

User-mode QEMU with symbolic execution. Superior performance when compared to S2E. Possibly a good
fit, but much much worse documentation when compared to S2E.

## S2E

https://github.com/S2E/s2e

System QEMU with symbolic execution. Essentially adds an instruction to symbolize a memory
range. Build GUI decompiler/(taint analyzer) using this as dynamic engine?)

~~Found https://s2e.systems/docs/StateMerging.html. This was the main advantage I saw in implementing
a custom (mini) symbolic executor and it turns out this is supported. (Also angr has something
similar https://docs.angr.io/extending-angr/state_plugins that I had missed earlier)~~

Actually, StateMerging requires custom "merge group" handling. This is obvious: our design doc uses
persistent data structures to solve the very same problem, i.e. we allow merging everywhere rather
than at the end of merge groups. But maybe we could add merge groups through the GUI decompiler
somehow?

Let's use this. I think we want to build some kind of GUI program that controls a S2E instance,
visualizes and works a bit like a symbolic-execution debugger? Future tasks:

- How powerful are S2E plugins?
- What plugin support is necessary?
- How do we control S2E?
