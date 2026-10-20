# Architecture

Status: initial hypotheses, not an implemented design.

## Layers

1. **Build driver:** invoke Cargo/compiler toolchains for the selected payloads.
2. **Linking and packaging:** resolve code, collect assets, and assemble one file.
3. **Host bootstrap:** enter through a mechanism the stock host can already run.
4. **Payload selection:** choose the supported architecture and OS implementation.
5. **Runtime boundary:** provide the filesystem, threading, networking, and other
   services expected by the program.

The initial Rust workspace implements only CLI help and version output. The
layering above identifies future responsibilities, not existing crates.

## Candidate payload designs

| Design | Benefit | Main work or limitation |
| --- | --- | --- |
| Full build per OS/CPU | Reuses normal Rust targets and native APIs | Larger artifact; still needs a universal bootstrap |
| Build per CPU plus portable runtime | Shares application code across OSes | Requires a suitable Rust target, `std`, ABI, and dependency integration |
| Portable bytecode plus runtimes | Architecture-neutral application payload | Each host still needs an executable runtime and bootstrap |

Treat these as alternatives to evaluate, not simultaneous commitments.

## Bootstrap boundary

Stock ELF, Mach-O, and PE loaders expect different headers. A header or a loader
embedded somewhere in a file is insufficient unless the host has a supported way
to reach it. OS detection inside application code happens after this boundary.

APE combines a Windows executable with Unix shell handling and loader machinery.
It is the first candidate to measure. Shell fallback is not equivalent to a
successful direct `execve`, `posix_spawn`, or Windows process launch. Embedded
extraction must preserve the distributed artifact and disclose its filesystem
requirements. A path that compiles a loader requires a compiler and does not meet
the no-prerequisites goal.

References:

- [APE specification](https://github.com/jart/cosmopolitan/blob/master/ape/specification.md)
- [Cosmopolitan toolchain caveats](https://github.com/jart/cosmopolitan/blob/master/tool/cosmocc/README.md)
- [Linux ELF loader](https://github.com/torvalds/linux/blob/master/fs/binfmt_elf.c)
- [Windows PE format](https://learn.microsoft.com/en-us/windows/win32/debug/pe-format)
- [Apple universal-binary header](https://github.com/apple-oss-distributions/xnu/blob/main/EXTERNAL_HEADERS/mach-o/fat.h)

## Rust boundary

Rust selects target-specific code before linking. A linker cannot reconstruct
branches excluded by `cfg(target_os)` or turn an already compiled native object
into code for a different instruction set.

Separate target builds preserve those compile-time decisions. A shared runtime
design instead needs a coherent target specification, standard library, ABI,
system constants, thread-local storage, and panic/unwind behavior. Native
dependencies and build scripts require their own compatibility assessment.

See [Rust conditional compilation](https://doc.rust-lang.org/reference/conditional-compilation.html)
and the experimental [cargo_cosmo project](https://github.com/ShieldedLabs/cargo_cosmo).
Do not infer compatibility from a successful link alone.

## Decisions still open

- Which launch paths are achievable without host changes?
- Can the bootstrap work on clean Apple Silicon systems without developer tools?
- How will Windows ARM64 select native code from the common artifact?
- How will signatures and notarization interact with the outer file and payloads?
- Is temporary extraction acceptable for the product, and where can it succeed?
- Which payload design best preserves useful Rust ecosystem compatibility?
- Which minimum OS versions and CPU feature baselines will be supported?

Resolve these with small experiments and decision records. A new linker is
justified when a measured requirement cannot reasonably be met with existing
linkers and a packaging stage.
