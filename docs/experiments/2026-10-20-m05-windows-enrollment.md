# M0.5 Windows runner enrollment

- Date: 2026-10-20.
- Source: `19e367414b32e2ed7213a66cfb72d2287ade2b78`.
- Result: `host-003` (Windows x86-64) enrolled; authenticated job delivery and report
  collection verified. Successful native payload execution remains unverified
  through the coordinator.

## Enrollment and provenance

For `host-003`, the remote operator reported Windows 11 Pro 25H2
(10.0.26200.9711), physical x86-64 hardware, and a non-elevated per-user Scheduled
Task requiring an interactive login. Formatting, pedantic Clippy, 16 behavioral
tests, and pinned native builds reportedly passed. These build/test claims came through the
user's enrollment summary; their logs were not independently inspected on the
coordinator machine.

The agent generated its own random credential locally. Only its token hash was
relayed for registration. The coordinator's Windows entry was updated and the
coordinator restarted while no jobs were pending. Other hosts' credential
entries were preserved. No additional network rule was needed.

The returned report identifies a Windows x86-64 agent with SHA-256
`380817afc8d1278834a9c544adc0cc6da4b68f1591e96df2c0b9efc2d4f75e3c`, matching
the enrollment summary. The declared baseline remains a developer workstation,
not a clean-machine qualification. Detailed host inventory stays in private
local evidence.

## Actual coordinator-driven result

The job published as `deploy-20261020-host-003-invalid` delivered the same 128
zero bytes used for the Mac and Linux negative controls. SHA-256 before and after
execution was
`38723a2e5e8a17aa7950dc008209944e898f69a7bd10a23c839d341e935fd5ca`.

Windows rejected the launch with:

```text
spawn failed: The file or directory is corrupted and unreadable. (os error 1392)
```

The report records `outcome: fail`, no exit code, no stdout/stderr, and no timeout.
This is the expected malformed-input failure and is retained unchanged. It
demonstrates authenticated delivery, attempted launch, hash collection, and report
return. It is not a successful native executable test.

The [sanitized evidence manifest](2026-10-20-m05-windows-enrollment-evidence.json)
contains the selected report fields, original source-report hash, and provenance
boundaries. Its `public_*` identifiers are substitutions; source hashes still
refer to the original private evidence. The full report and identity mapping
remain private. See the [publication policy](../public-evidence.md).

## Open verification

The Windows-built control is reported as 198144 bytes, SHA-256
`3c4e7d973f6e3686b19f58a4aa26f6c6dac2a9a9763d909189763b52ce7b8a1d`.
It remains on Windows and has not been transferred to the coordinator. Acquire
that exact artifact and verify its size/hash before queuing Unicode/stdio,
raw-byte, and timeout controls. Then qualify Windows supervision and recovery.
Strict launch behavior, clean/offline/security baselines, and portable-artifact
acceptance remain open. M0.5 is not complete.
