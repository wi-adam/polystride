# Validation contract

## Initial target matrix

These are intended targets, not supported or tested targets. Minimum OS versions
remain to be chosen. The native developer CLI is outside this artifact matrix.
Host-native lab agents and their control programs are also outside this matrix.

| OS | CPU | Portable-artifact execution | Native execution |
| --- | --- | --- | --- |
| Linux | x86-64 | Not tested | Not tested |
| Linux | ARM64 | Not tested | Not tested |
| macOS | x86-64 | Not tested | Not tested |
| macOS | ARM64 | Not tested | Not tested |
| Windows | x86-64 | Not tested | Not tested |
| Windows | ARM64 | Not tested | Not tested |

## M0.5 infrastructure qualification

Track lab qualification independently of portable-artifact acceptance.
[The lab runbook](lab.md) describes the implementation and remaining gates.

| Public host ID | Environment | Lab evidence | Qualification |
| --- | --- | --- | --- |
| `host-001` | macOS ARM64 developer machine | TLS delivery, native controls, failed controls, LaunchAgent deployment and reconnection after restart | Developer baseline; implicit shell fallback observed |
| `host-002` | Linux x86-64 staging host | TLS delivery, native controls, failed controls, systemd deployment and reconnection after restart | Existing host, not a clean baseline; strict launch qualification open |
| `host-003` | Windows x86-64 developer machine | Resident runner enrolled; authenticated delivery and malformed-input report collected | Positive native controls and Windows restart qualification pending |
| Unassigned | Linux ARM64 | Not tested | Agent deployment and native controls open |
| Unassigned | Windows ARM64 | Not tested | Agent deployment and native controls open |
| Unassigned | macOS x86-64 | Not tested | Agent deployment and native controls open |

Public IDs are publication aliases, not hostnames or necessarily live protocol
IDs. Their operational mapping stays private. Follow
[the public evidence policy](public-evidence.md) for reports and manifests.

These results do not certify a strict direct-launch backend or clean host
baselines. See the [initial lab report](experiments/2026-10-20-m05-local-tls.md)
and the [Mac/Linux deployment report](experiments/2026-10-20-m05-host-deployment.md).
The [Windows enrollment report](experiments/2026-10-20-m05-windows-enrollment.md)
records the subsequent Windows connection and its remaining verification gates.

## Record per experiment

- Date, Polystride source commit, upstream revisions, and exact build commands.
- Opaque public host IDs; keep their operational mapping private.
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
Do not commit credentials, operational host identities/addresses, or detailed
private inventories. Follow [the public evidence policy](public-evidence.md).
Keep raw evidence immutable; label public ID substitutions and source hashes
explicitly in sanitized extracts.

The [initial M0.5 report](experiments/2026-10-20-m05-local-tls.md) concerns test
infrastructure only. No portable-artifact experiment has been accepted.
