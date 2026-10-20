# Goals

## North star

Produce one immutable executable file that runs on Linux, macOS, and Windows
across x86-64 and ARM64, without platform selection by the user, prerequisite
installation, network access at launch, or OS configuration changes.

Native execution on each target is the objective. A demonstration that needs
emulation, an explicit shell command, an external loader, development tools, or
special permissions must state those requirements and does not satisfy the full
goal. Normal OS authorization prompts and the steps required after a browser
download must also be documented.

The operating systems' existing executable loaders impose a real feasibility
boundary. Preserve the objective and record blockers; do not silently redefine
"just works" to mean a narrower launch path.

## Requirements

| ID | Requirement | Acceptance evidence |
| --- | --- | --- |
| G1 | Identical artifact on every target | Same SHA-256 and size before and after execution on each host |
| G2 | Linux, macOS, Windows; x86-64 and ARM64 | Actual execution on all six combinations with OS versions recorded |
| G3 | No user-installed runtime, compiler, loader, or emulator | Clean-machine inventory and offline first-run evidence |
| G4 | Predictable launch behavior | Separate shell, direct process-spawn, and desktop-launch results |
| G5 | Useful application behavior | Arguments, Unicode, stdio, exit status, files, time, threads, and local TCP probes |
| G6 | Preserve ordinary Rust development | A documented build command and an explicit crate/FFI compatibility boundary |
| G7 | Inspectable artifacts and results | Payload inventory, pinned inputs, source revision, hashes, and limitations |
| G8 | Viable distribution | Measured signing, notarization, download/quarantine, and executable-permission behavior |

All requirements are currently **unverified**. See
[the validation contract](docs/validation.md) for how evidence is recorded.

## Milestones

### M0 — Repository foundation

- [x] Name the project and establish a Cargo workspace.
- [x] Record goals, architecture hypotheses, and validation criteria.
- [x] Provide a pinned toolchain and local quality commands.

M0 establishes the development scaffold; it establishes no portable execution
claim.

### M1 — Prove the bootstrap boundary

- [ ] Select and pin an APE baseline.
- [ ] Build one minimal artifact and preserve its original bytes.
- [ ] Run that same artifact on clean hosts for all six target combinations.
- [ ] Test shell launch and direct process spawning independently.
- [ ] Record native versus translated CPU execution.
- [ ] Record extraction, compiler use, disk writes, network use, and prerequisites.
- [ ] Identify exact failures and write a go/no-go decision for the bootstrap.

An upstream C hello-world is acceptable for isolating this milestone's loader
question. It must be identified as upstream evidence, not Rust or Polystride
implementation evidence. If the strict goal is blocked, document the blocker and
the closest demonstrated behavior without calling the goal complete.

### M2 — Build a Rust vertical slice

- [ ] Build a minimal Rust program into the selected artifact design.
- [ ] Automate the process through the Polystride CLI.
- [ ] Inspect the artifact's payloads and architecture coverage.
- [ ] Verify one unchanged artifact across the supported matrix.
- [ ] Document any `no_std`, panic, TLS, FFI, or crate restrictions.

Start with [examples/hello.rs](examples/hello.rs) or a smaller `no_std` probe if
the chosen runtime requires it. Ordinary target-specific builds are controls.

### M3 — Qualify a useful CLI runtime

- [ ] Exercise the G5 behavior probes, including failure paths.
- [ ] Decide between OS-specific payloads and a shared portable runtime.
- [ ] Establish a supported Rust `std` and dependency compatibility policy.
- [ ] Preserve exit codes, arguments, working directory, and child-process behavior.
- [ ] Measure startup latency and artifact size against native controls.

### M4 — Qualify distribution

- [ ] Test realistic download and launch paths with platform security enabled.
- [ ] Resolve signing and notarization for the selected design.
- [ ] Test read-only artifact locations and restricted temporary directories.
- [ ] Automate regression checks across supported OS versions and CPUs.
- [ ] Select a license, public hosting, and a release policy.

## Initially deferred

- Arbitrary future architectures and operating systems.
- GUI frameworks, GPU APIs, drivers, and privileged services.
- Compatibility with every Rust crate or arbitrary native dynamic library.
- A new optimizer, compiler backend, or general-purpose linker.
- Reproducible builds across different build hosts as a release guarantee.

Distributing identical bytes is required now as a goal; independently rebuilding
identical bytes on different hosts is a separate later property.

## Next task

Design and run the M1 bootstrap experiment. Record the exact host baselines and
launch methods before investing in a new executable format or linker.
