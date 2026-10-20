# Polystride

**One executable. Many platforms.**

Polystride is a Rust project exploring how to produce one byte-for-byte identical
executable that runs on Linux, macOS, and Windows across CPU architectures.

The goal is a single file that users can download and run without choosing a
platform build, installing a runtime, or configuring their operating system.
Native execution on x86-64 and ARM64 is the initial target.

## Status

**M0.5 execution laboratory in progress.** The `polystride` CLI remains a scaffold.
The new `polystride-lab` component delivers and runs ordinary test executables
through resident agents that connect back to a TLS coordinator. This is an initial
research harness; it does not establish portable executable support.

The macOS ARM64 and Linux x86-64 lab hosts have completed native controls and
service-restart checks. The Windows x86-64 runner is enrolled and has returned
a malformed-input report; positive and timeout controls through the coordinator
remain pending. Three other OS/CPU targets remain unenrolled. See the
[infrastructure matrix](docs/validation.md#m05-infrastructure-qualification)
for public host IDs and evidence.

Before M1, finish qualification of the testing infrastructure and remaining
hosts. One file running through a shell, an installed loader, or CPU emulation
is useful evidence, but each qualification must be recorded.

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
- [Execution lab design and runbook](docs/lab.md)
- [Public evidence and host privacy](docs/public-evidence.md)
- [Initial architecture decision](docs/decisions/0001-start-with-a-driver.md)
- [Contributor and agent guidance](AGENTS.md)

## Development

The development toolchain is pinned to Rust 1.93.0. Rustup will use
`rust-toolchain.toml`; install that toolchain and its rustfmt/clippy components
before working offline. The scaffold CLI has no third-party Rust dependencies;
the lab uses locked dependencies for TLS, hashing, JSON, and atomic file
publication. Fetch the lockfile's dependencies once before working offline.

```sh
cargo run --locked --offline -- --help
cargo run --locked --offline -- --version
cargo fmt --all -- --check
cargo clippy --locked --offline --workspace --all-targets -- -D warnings
cargo test --locked --offline --workspace
```

With [just](https://github.com/casey/just) installed, `just check` runs formatting,
Clippy (including the workspace's pedantic lints, with warnings denied), and
behavioral tests, including actual execution of host-native controls.
`just lab-smoke` exercises the coordinator and agent over local TLS and retains
raw evidence under `artifacts/`; it also requires Python 3 and OpenSSL.
Builds of these tools are normal host-native binaries, not proof of the
portable-artifact goal. Use `cargo run -p polystride-lab -- --help` for lab commands.

## Repository layout

```text
crates/polystride/  CLI entry point
crates/polystride-lab/  resident execution agent and coordinator
docs/              architecture, validation, and decision records
examples/          portability probes, native lab control, and sample lab configs
scripts/           repeatable local lab smoke experiment
GOALS.md           project objectives, milestones, and deferred scope
```

Generated binaries and local evidence go under `artifacts/`, which is ignored.
Commit concise, sanitized experiment reports with hashes, exact source
revisions, and opaque public host IDs. Keep operational identities and their
mapping outside tracked files; follow the [publication rules](docs/public-evidence.md).

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

This research repository is publicly hosted on GitHub. Package publication is
disabled; a license and release policy have not yet been selected.
