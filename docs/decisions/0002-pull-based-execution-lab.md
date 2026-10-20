# ADR 0002: A resident pull agent for execution evidence

- Date: 2026-10-20
- Status: accepted for M0.5's initial implementation; fleet qualification open

## Context

M1 needs repeatable execution evidence from different OS/CPU combinations.
Cross-compilation, emulation, and successful CI builds cannot supply all of that
evidence. Manual copying and terminal transcripts also make it easy to lose
artifact identity, failed controls, or launch-path qualifications.

The user chose a resident agent that connects back, rather than SSH invocation.
The agent must remain separate from the executable under test.

## Decision

Add `polystride-lab`, a Rust component with coordinator, agent, queue, and status
commands. Build the infrastructure for each host normally. Deliver the exact
artifact bytes in an immutable job to a named host. Verify SHA-256 at receipt,
after staging, and after execution. Keep byte output, status, timing, and
qualifications in a versioned JSON report.

Agents initiate TLS connections and poll for work. Authenticate the coordinator
against an explicitly configured CA and server name; authenticate each agent with
a unique random token whose hash is stored by the coordinator. The first protocol
is a bounded, length-prefixed JSON request/response over TLS, not HTTP. Reuse
rustls and ring; do not implement encryption or certificate verification.

The operator queues through the coordinator's local filesystem. There is no
remote job-submission endpoint or arbitrary remote shell. A single coordinator
and a single agent per state directory hold OS file locks. Persist immutable jobs,
assignment markers, receipts, and reports with atomic publication.

Redeliver unacknowledged assignments, but never automatically re-execute a job
whose receipt exists. Resubmit its saved report, or record `blocked` if an agent
restart leaves execution ambiguous. A deliberate retry is a new job ID; preserve
the original evidence. Do not claim distributed exactly-once execution.

## Consequences

- Agents require outbound connectivity, with no inbound service on test hosts.
- Host-native agent builds do not weaken the identical-artifact requirement.
- We can test orchestration on VMs, then qualify platform behavior on appropriate
  physical hosts. Machine type and translation status remain separate facts.
- File-based storage and sequential requests fit the initial small research lab;
  a scheduler, database, HTTP transport, and remote operator API are deferred.
- The first executor uses `std::process::Command`. It is explicitly unqualified
  for strict direct launch: the pinned Rust Unix implementation includes
  `execvp`/`posix_spawnp` paths. A fallback-detection control is part of the local
  smoke experiment. Backend qualification must precede M1.
- The first executor supervises only the direct child and has soft output limits.
  It is for trusted single-process probes; process trees and hard resource
  containment are required before unattended general workloads.
- The agent is instrumentation installed on a host, not evidence that an
  executable requires a runtime. We still need reset baselines and tests showing
  the artifact cannot find or use tools, caches, or network access provided by
  that instrumentation.
- Reusable startup packaging, full supervisor qualification, baseline discovery,
  actual workload architecture, tracing, offline execution, and real
  download/desktop launch remain open gates. Individual host supervision is now
  deployed; the [validation matrix](../validation.md) tracks its evidence.

## References

- [Pinned Rust 1.93.0 Unix process implementation](https://github.com/rust-lang/rust/blob/1.93.0/library/std/src/sys/process/unix/unix.rs)
- [Rust process API and Windows process creation](https://doc.rust-lang.org/std/process/index.html)
- [rustls 0.23.42](https://docs.rs/rustls/0.23.42/rustls/)
- [Windows on Arm emulation and architecture detection](https://learn.microsoft.com/en-us/windows/arm/apps-on-arm-x86-emulation)
