# Validation contract

## Initial target matrix

These are intended targets, not supported or tested targets. Minimum OS versions
remain to be chosen. The native developer CLI is outside this artifact matrix.

| OS | CPU | Portable-artifact execution | Native execution |
| --- | --- | --- | --- |
| Linux | x86-64 | Not tested | Not tested |
| Linux | ARM64 | Not tested | Not tested |
| macOS | x86-64 | Not tested | Not tested |
| macOS | ARM64 | Not tested | Not tested |
| Windows | x86-64 | Not tested | Not tested |
| Windows | ARM64 | Not tested | Not tested |

## Record per experiment

- Date, Polystride source commit, upstream revisions, and exact build commands.
- Build host, compiler/linker versions, inputs, artifact size, and SHA-256.
- Test host OS build, CPU architecture, baseline CPU features, and installed tools.
- Whether any host configuration differs from a clean installation.
- Exact invocation and whether a shell or external helper participates.
- Exit status, stdout/stderr, expected behavior, and observed behavior.
- Native execution versus CPU translation/emulation.
- Extracted files, cache state, writable-directory requirements, and network use.
- SHA-256 before and after execution; test an initially empty helper cache.
- Signing/quarantine state, download method, permission changes, and prompts.

Use `pass`, `fail`, `blocked`, or `not tested` for each check. Record failures as
well as successes. A build or structural inspection is not execution evidence.
An emulated result is not evidence of native execution on that CPU.

## Launch paths

Record these separately:

1. A terminal shell launches the executable.
2. A native parent process launches it without shell fallback.
3. A desktop user opens the downloaded file, where applicable.
4. An explicit shell or loader command launches it as a diagnostic fallback.

A success in one row must not imply success in the others.

## Functional probes

Begin with stdout and an explicit exit status. Then add argument and Unicode
handling, environment and working directory, stderr/pipes, file create/read/delete
inside an isolated temporary directory, monotonic time, threads/TLS, local TCP,
child processes, and failure-path error reporting. Compare against a native
control program on each host.

Keep the distributed executable read-only where possible. Exercise it offline,
without preinstalled custom helpers, and with an initially empty extraction
cache. Record restrictions rather than changing host security settings to obtain
a passing result.

## Evidence storage

Store generated binaries and raw local logs in ignored `artifacts/` directories.
Commit concise reports under `docs/experiments/` when experiments exist, including
hashes, source revisions, commands, host baselines, failures, and qualifications.
Do not commit credentials or secret-bearing host inventories.

No experiment reports exist yet.
