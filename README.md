# Polystride

**One executable. Many platforms.**

Polystride is a Rust project exploring how to produce one byte-for-byte identical
executable that runs on Linux, macOS, and Windows across CPU architectures.

The goal is a single file that users can download and run without choosing a
platform build, installing a runtime, or configuring their operating system.
Native execution on x86-64 and ARM64 is the initial target.

## Status

**Project scaffold only.** The workspace builds a small CLI with help and version
output. It does not yet link, package, or launch portable executables. No
cross-platform artifact has been validated.

The first milestone is to establish what can actually launch on clean systems.
One file running through a shell, an installed loader, or CPU emulation is useful
evidence, but each qualification must be recorded.

## Approach

Start with a Rust build driver and packager around existing compilers and linkers.
Investigate Cosmopolitan / Actually Portable Executable (APE) as the first
bootstrap candidate. Let experiments determine whether we need a new container,
loader, portable runtime, or custom linker.

We will evaluate both complete OS/CPU-specific payloads and shared code backed by
a portable runtime. No final executable format or runtime has been selected.

See:

- [Goals and milestones](GOALS.md)
- [Architecture and unresolved questions](docs/architecture.md)
- [Validation contract and target matrix](docs/validation.md)
- [Initial architecture decision](docs/decisions/0001-start-with-a-driver.md)
- [Contributor and agent guidance](AGENTS.md)

## Development

The development toolchain is pinned to Rust 1.93.0. Rustup will use
`rust-toolchain.toml`; install that toolchain and its rustfmt/clippy components
before working offline. The scaffold has no third-party Rust dependencies.

```sh
cargo run --locked --offline -- --help
cargo run --locked --offline -- --version
cargo fmt --all -- --check
cargo clippy --locked --offline --workspace --all-targets -- -D warnings
cargo test --locked --offline --workspace
```

With [just](https://github.com/casey/just) installed, `just check` runs formatting,
Clippy, and the test harness. There are no behavioral tests yet; the CLI is a
scaffold. Builds of this CLI are normal host-native binaries, not proof of the
portable-artifact goal.

## Repository layout

```text
crates/polystride/  CLI entry point
docs/              architecture, validation, and decision records
examples/          small source programs for future portability experiments
GOALS.md           project objectives, milestones, and deferred scope
```

Generated binaries and local evidence go under `artifacts/`, which is ignored.
Commit concise, sanitized experiment reports with hashes and exact source
revisions when experiments begin.

## Prior art

- [Cosmopolitan](https://github.com/jart/cosmopolitan)
- [APE specification](https://github.com/jart/cosmopolitan/blob/master/ape/specification.md)
- [Cosmopolitan toolchain and launch caveats](https://github.com/jart/cosmopolitan/blob/master/tool/cosmocc/README.md)
- [Rust APE experiments: cargo_cosmo](https://github.com/ShieldedLabs/cargo_cosmo)
- [Rust target configuration](https://doc.rust-lang.org/reference/conditional-compilation.html)
- [Wasmtime](https://docs.wasmtime.dev/introduction.html)

These are research references, not vendored dependencies or verified Polystride
capabilities. Pin upstream revisions when an experiment relies on them.

## Distribution

This is a local research repository. Package publication is disabled. An
open-source license and public hosting destination have not yet been selected.
