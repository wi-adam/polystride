# M0.5 Mac and Linux host deployment

Follow-up: [Windows enrollment](2026-10-20-m05-windows-enrollment.md) was
subsequently completed, with positive Windows control execution still pending.
The observations below describe the earlier Mac/Linux deployment stage.

- Date: 2026-10-20.
- Source: `58617924785f92b4ac72d9018d857f953c5ff3e1`.
- Result: coordinator and resident agents deployed on `host-001` and `host-002`
  with delivery, execution, evidence collection, and service reconnection
  demonstrated. Windows was not yet deployed at this experiment stage.
- Scope: host-native infrastructure controls, not portable-artifact acceptance.
  M0.5 remains incomplete.

## Builds and host baselines

Both builds use the committed source and locked dependencies, with Rust 1.93.0.
The Mac release build uses the local pinned toolchain. The Linux release build
uses an exact Git archive in the official Rust container:

```text
rust:1.93.0-slim-bookworm
rust@sha256:776861219cd851131c1cec3bbd7cbeb16b99a794048097eb69ad9682a8ed0d57
```

The container ran with `--platform linux/amd64` on the ARM64 Mac, two CPUs and
3 GiB memory. This is an emulated **build**, followed by execution on the physical
Linux host. No compiler or container runtime was added to that host.

Build commands inside the respective build environments:

```sh
cargo build --release --locked -p polystride-lab
rustc --edition=2024 -C debuginfo=0 examples/lab-control.rs -o OUTPUT
```

The Linux Cargo invocation additionally used `-j 2`. `OUTPUT` identifies a
different host-native control on each platform.

| Public host ID | Observed baseline | Qualification |
| --- | --- | --- |
| `host-001` | macOS ARM64; macOS 26.5.2, build 25F84; Darwin 25.5.0 | Physical developer machine; tools and network available |
| `host-002` | Linux x86-64; Ubuntu 26.04.1 LTS; Linux 7.0.0-38-generic; `systemd-detect-virt` reports `none` | Existing physical staging host; dedicated service account; network available |

These opaque publication IDs are separate from private operational identities.

`file` identifies the controls as ARM64 Mach-O and x86-64 ELF respectively.
Their targets match the execution hosts. The runner itself still records workload
translation as unknown: it does not automatically audit translation, CPU features,
security state, or clean-machine prerequisites. Neither host is a clean baseline.

## Installed services and network scope

The coordinator and agent on `host-001` run as user LaunchAgents with `RunAtLoad`,
`KeepAlive`, and private configuration, token, key, and evidence directories.
They start in the user's login session; the coordinator depends on that Mac
remaining awake and the session available.

The agent on `host-002` runs as a dedicated unprivileged account under an enabled
systemd service with automatic restart. Its writable state is isolated from
root-owned configuration and executable files. Service limits include two CPUs, 1 GiB
memory, 64 tasks, and whole-service control-group cleanup when the service stops.
These service settings do **not** implement per-job descendant termination or
hard output/disk limits in the runner.

The initial TCP probe from `host-002` to the coordinator on `host-001` timed out.
After explicit approval, one firewall rule allowed only that source, destination,
and coordinator TCP port. The probe then connected, followed by authenticated
TLS job delivery and returned reports. Operational addresses, network topology,
and the saved firewall screenshot remain in private local deployment evidence.

No SSH forwarding policy or host firewall setting was changed. The coordinator
uses a private CA, server-name verification, and a different random token for
each enrolled host. Certificate renewal details remain in private operational
records.

## Controls and restart verification

Each host received its own native control plus the same 128-byte malformed
payload. Public job labels use the prefix `deploy-20261020-`, followed by the
opaque host ID and case; the original protocol identifiers remain private.

| Case on each host | Expected observation | Actual result |
| --- | --- | --- |
| `echo` | Literal Unicode argument, exact stdout/stderr, exit 7 | Recipe pass on both |
| `bytes` | Raw stdout bytes `00 ff 0a`, exit 0 | Recipe pass on both |
| `timeout` | Ten-second sleep exceeds an 80 ms deadline | Retained recipe failure on both; timeout recorded |
| `invalid` | Malformed executable does not satisfy a successful-exit recipe | Retained failure on both; Linux spawn error, macOS exit 126 |
| `restart` | Fresh byte-output job after restarting coordinator and both agents | Recipe pass on both |

All ten reports retain matching artifact hashes and sizes before and after
execution. Agent hashes in the reports match the installed executables. After
the service restarts, the original eight report files had unchanged SHA-256
values, and two new jobs completed successfully. Restarts occurred with no jobs
in flight; this does not qualify arbitrary mid-execution interruption or host
reboot recovery. Boot/login registration was inspected, but neither host was
rebooted for this experiment.

macOS exit 126 for malformed input is consistent with the shell fallback
previously demonstrated by the independent sentinel. It is not relabeled as a
kernel spawn rejection. The [earlier failed fallback control](2026-10-20-m05-local-tls.md)
still prevents strict no-fallback launch qualification.

## Artifact identity and retained evidence

| Artifact | SHA-256 |
| --- | --- |
| Mac agent/coordinator | `8db833227fc1cbc98b376e8585a52ca9c677e003a41d88bcf59cf37f47e96c13` |
| Linux agent | `62cecaef59fc47216f611c9f6119d6c4494b462a365bd0e1f0d8b2c6968ccac8` |
| Mac native control, 519336 bytes | `2ae95185f018a5c6e9af9578159f40867d5366b365351542ac1a21fb9e0c595e` |
| Linux native control, 3962872 bytes | `c1f64fc92093be07b0b44f3e67dbbc56821ae91f1abfedf6fa5c93cab56c0063` |
| Malformed control, 128 bytes | `38723a2e5e8a17aa7950dc008209944e898f69a7bd10a23c839d341e935fd5ca` |

The [sanitized evidence manifest](2026-10-20-m05-host-deployment-evidence.json)
records original source-report/job hashes and selected observations. Its
`public_*` IDs are substitutions, and its `source_*_sha256` digests refer to the
original private evidence, not the edited extract. See the
[publication policy](../public-evidence.md). The malformed-input diagnostic on
`host-001` contains a private absolute path, so its public stderr bytes are
explicitly omitted while the original byte count and captured-byte digest remain.
Raw reports and receipts remain
in the coordinator's private state and each agent's state. Build outputs,
installation scripts, recipes, and firewall screenshots are retained locally in
`artifacts/deployment-2026-10-20/`. Do not publish that directory wholesale:
enrollment material and private configuration are separate from public evidence.

No Rust source changed during deployment. The deployed source previously passed
formatting, pedantic Clippy with warnings denied, all 17 behavioral tests, and
the local TLS smoke experiment. This report adds actual Linux execution and
service deployment evidence; it does not broaden those local quality checks into
Windows or six-platform acceptance.

## Remaining work

At this stage, Windows enrollment was the next deployment task; the
[follow-up report](2026-10-20-m05-windows-enrollment.md) records its completion
and outstanding native controls. Linux ARM64 and the remaining two matrix targets
remain unenrolled. Strict launch backends, per-job process-tree containment,
hard resource limits, clean/offline/security baselines, and broader failure
recovery controls remain open. No portable executable has been tested.
