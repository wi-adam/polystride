# M0.5 execution laboratory

Status: initial Rust implementation for trusted single-process probes.
The six-host fleet and the full M0.5 acceptance gates are not qualified.
The product's identical-artifact goal is unchanged.

## Topology

```text
Operator
  queue recipe + exact binary bytes
        |
        v
Coordinator: immutable jobs / assignments / results
        ^
        | outbound TLS polling and report delivery
        |
Resident host-native agents
  Linux x86-64 | Linux ARM64 | Windows x86-64 | macOS ARM64
  then macOS x86-64 | Windows ARM64
```

One `polystride-lab` executable supplies the coordinator and agent commands.
Use an ordinary build for each OS/CPU; these infrastructure executables need not
be identical. Only the artifact being evaluated must have identical bytes.
There is no inbound listener on agents, SSH requirement, remote command shell,
or runtime dependency on Python/OpenSSL for agents.

The coordinator uses a small versioned protocol over TLS, **not HTTPS**.
Use a private lab network or private tunnel. The initial sequential server has
15-second socket idle timeouts, not total connection deadlines or public-service
rate limits. Do not place it behind an HTTP proxy.

## Hosts and acceptance

Begin with the user's four proposed targets and extend to all six in G2.
Provision dedicated test accounts and a reproducible baseline for each host.

| Tier | Use | Qualification |
| --- | --- | --- |
| Developer host | Build, unit tests, local orchestration smoke | Tools/caches present; no clean-host claim |
| Same-architecture VM | Repeatable OS baseline and regression controls | Record guest/hypervisor; not physical-host evidence |
| Physical host | Loader, CPU, security and distribution acceptance | Record actual hardware, OS build and security state |
| Emulated/translated process | Diagnostic comparisons | Never count as native CPU acceptance |

VMs are useful and need not be excluded. They do not cover every physical-machine
or distribution question. Also, physical ARM hardware does not prove a workload
ran natively. For example, Windows can translate x64 programs on ARM; see
[Microsoft's architecture-detection guidance](https://learn.microsoft.com/en-us/windows/arm/apps-on-arm-x86-emulation).

The current `baseline` is operator-declared: OS/version, hardware architecture,
physical/VM/emulator/unknown, baseline record ID, and notes. The report separately
records the agent's compile target and binary hash. Workload translation status
remains **unknown**. Keep detailed baseline records outside Git if they contain
sensitive inventories.

Before M1, add verified host discovery, native controls on each target, strict
launch backends, whole-process-tree supervision, clean reset procedures, and
evidence of offline/empty-cache behavior. Startup supervision must also be
qualified for each OS. Neither `cargo check --target` nor a VM-only run closes
physical/native acceptance.

## Local end-to-end experiment

```sh
just check
just lab-smoke
```

The smoke script needs Python 3 and OpenSSL on the developer machine. It builds
the agent and a native Rust control, makes short-lived local certificates and a
random host token, starts a localhost coordinator, runs the actual agent, and
stops the coordinator when done. Nothing is installed as a service.

It retains positive output/exit/hash evidence, a timed-out control, malformed
executable evidence, a no-shebang shell-fallback sentinel, and rejected TLS-name
and credential controls. It records source and executable hashes under
`artifacts/lab-smoke/run-*/`. Keys and tokens stay in that ignored private
directory; do not upload the whole directory. Share only reviewed summaries.
The generated CA/server certificates expire after two days.

The behavioral tests also exercise Unicode/raw bytes, environment and cwd, file
operations, output overflow, corrupted artifacts, executable replacement on Unix,
duplicate reports/jobs, restart receipts, host credential scope, and state locks.
The test fixture is [examples/lab-control.rs](../examples/lab-control.rs).

## Deploy a research agent

Build for the intended host using the pinned toolchain and lockfile:

```sh
cargo build --release --locked -p polystride-lab
```

Build on that platform or use a separately verified cross-build toolchain. Copy
only the resulting `polystride-lab`/`polystride-lab.exe`, agent configuration, CA
certificate, and token to the execution host. A Rust compiler is a build-time
dependency; deployed agents do not invoke Cargo or rustc. Runtime/linker
prerequisites still need measurement on each clean target.

Start from the example [coordinator](../examples/lab/coordinator.json),
[agent](../examples/lab/agent.json), and [recipe](../examples/lab/recipe.json)
files. Copy them into an ignored deployment directory and replace placeholders.
All configuration paths resolve relative to the process working directory;
absolute paths are recommended for deployed agents.

1. Provision a server certificate with the coordinator's DNS name in its SAN and
   a private CA trusted by the agents. Install only the public CA certificate on
   agents. Keep the server key on the coordinator.
2. Generate a different random token for each host, for example 32 random bytes
   encoded as hex. Put it in that agent's `token_file`, with owner-only access.
   Put SHA-256 of the token's UTF-8 text **without trailing whitespace** in the
   coordinator's `hosts` map. Tokens are not command-line arguments.
3. Set a reachable bind address on the coordinator, the matching TLS name on each
   agent, and private writable state directories. Preserve the token, directory,
   and baseline identity across normal agent restarts.
4. Run these as dedicated, unprivileged test accounts:

```sh
polystride-lab coordinator artifacts/lab/coordinator.json
polystride-lab agent artifacts/lab/agent.json
```

The agent remains running, polls at its configured interval, executes one job at
a time, and retries connection/report delivery failures. `agent CONFIG --once`
performs one poll and, if assigned, one execution/report attempt, then exits.
Its exit status reflects transport/agent operation; read the report's `outcome`
for whether the tested binary passed.

The current executable is a foreground process. Native service installers,
systemd/launchd units, and Windows SCM integration are not implemented. Do not
register it directly as a Windows service and assume it implements SCM callbacks.
Use foreground sessions for initial host qualification; add and test the
appropriate supervisor before unattended operation.

Host enrollment and token rotation are local configuration edits; restart the
coordinator to load them. There is no auto-enrollment or remote submission API.

## Queue and inspect a run

Build the native control for the test host first. For example, on that build host:

```sh
rustc --edition=2024 examples/lab-control.rs -o artifacts/lab/native-control.exe
```

After setting the recipe's source revision and host ID:

```sh
polystride-lab queue artifacts/lab/coordinator.json artifacts/lab/recipe.json artifacts/lab/native-control.exe
polystride-lab status artifacts/lab/coordinator.json
```

`queue` runs on the coordinator machine and snapshots the artifact into the
immutable job. It prints its SHA-256 and size. Subsequent edits to the original
file do not alter that job. Use different job IDs for different hosts and retry
attempts. For a portable experiment, queue the **same source artifact file** to
every host and compare the recorded hashes; do not rebuild between hosts.

The sample recipe expects raw stdout bytes `[0, 255, 10]` from the native control.
Output fields are byte arrays, preserving non-UTF-8 data. Args are an argument
vector, never a shell command string. `env` is explicit; parent environment is
cleared. HOME/USERPROFILE and temporary-directory variables are overwritten with
per-attempt directories. Windows SystemRoot is inherited for OS operation.
No agent token or coordinator configuration is injected into the child.

State lives in:

- Coordinator: `jobs/ID.json`, `assigned/ID.json`, `results/ID.json`.
- Agent: `ID/receipt.json`, `ID/job.json`, `ID/report.json`, staged `payload.exe`,
  `stdout.bin`, `stderr.bin`, and separate `work/`, `home/`, `tmp/` directories.

Jobs embed exact artifact bytes, recipe, protocol version, and SHA-256. Reports
bind to the digest of the complete job, not just its executable. Reports include
agent identity, declared baseline, received/staged and post-execution artifact
hash/size, output, status, timings, backend, and qualifications. `artifact_staged`
and optional `started_unix_ms` distinguish a real launch attempt from an
interrupted/unstarted receipt. Before staging, the "before" hash refers to the
verified received bytes. A recovered attempt has no claimed start time.

A `pass` means only that the recipe's exit/output expectations matched and the
artifact stayed unchanged without hitting the configured limits. It does not
certify native CPU execution, loader behavior, distribution, or platform support.
Missing result files remain queued/assigned, never implicitly passing.

## Disconnects, crashes, and retries

The coordinator retains an assignment before delivery. If the connection drops,
the next poll returns the same job. The agent writes a receipt before execution:

- If it has a complete local report, it sends that report again without running
  the payload again. The coordinator accepts identical duplicate reports.
- If the receipt exists but the report is missing, the agent records `blocked`
  with an unknown execution state. It does not repeat an ambiguous execution.
- Reusing a job ID with different content or replacing existing evidence fails.

Duplicate-execution protection depends on preserving the agent's state directory.
Do not enroll two independent agents with the same host identity, discard their
receipts, or copy credentials to a second machine. Loss of agent state requires
manual reconciliation of pending assignments before reconnecting.

Complete files are synced and published without replacement. OS locks prevent
two processes sharing a state directory. Recovery is designed for process crashes
and broken connections; filesystem durability through sudden power loss has not
been qualified. Do not delete receipts to force a retry. Inspect/clean the host,
preserve the original result, and queue a new ID. A lost or retired agent leaves
its assignment visibly pending; automatic reassignment and cancellation are
deferred.

## Current limits and the next M0.5 work

These limits are also recorded in reports:

- **Launch semantics:** `rust_std_process_command` is the only backend. It is
  not certified as strict no-shell-fallback launch. The pinned Unix Rust code
  includes `execvp`/`posix_spawnp`; preserve the fallback sentinel results. Add
  OS-native direct-spawn, terminal-shell, explicit-loader, and desktop paths as
  distinct test cases. Never rename this backend "direct" based on API shape.
- **Containment:** only the immediate child is terminated on timeout or output
  overflow. Descendants can outlive it or an agent crash. Run only trusted,
  single-process controls until process groups/OS jobs and crash cleanup are
  implemented. The working directory and cleared environment are not a sandbox.
- **Limits:** artifacts up to 8 MiB; protocol frames up to 40 MiB; recipe timeout
  1–300,000 ms; retained output up to 64 KiB per stream. Output is written to files
  and checked every 10 ms, so disk growth is a soft limit. Add hard filesystem,
  CPU/memory limits and fleet capacity controls before larger/untrusted workloads.
- **Filesystem and security:** a fresh payload path is used per job, Unix mode is
  explicitly set to `0500`, and byte integrity is checked. This does not recreate
  browser quarantine, signatures, original permissions, or read-only mount rules.
  Capture files, agent credentials, and state share an OS account; trusted
  payloads can access that account's files.
- **Observability:** no automatic OS-build/CPU-feature inventory, workload
  architecture detection, last-seen/heartbeat inventory, full filesystem/network
  tracing, cache audit, offline isolation, or desktop interaction yet.
- **Evidence:** no normalization from unknown to native/clean/offline. Raw output
  and operator-supplied environment may be sensitive; review summaries before
  committing them. Retention/cleanup is manual.

Finish strict launch semantics and process supervision first, then qualify
deployment/recovery on the first four hosts and extend to the remaining two.
The M1 bootstrap is still gated on the verification environment.
