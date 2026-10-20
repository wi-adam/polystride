# Contributor and agent guidance

- Read `README.md`, `GOALS.md`, and `docs/validation.md` before substantive changes.
- Subagents are allowed.
- Preserve the goal of one identical artifact; record qualified demonstrations
  separately from full acceptance.
- Keep scaffolding, successful builds, structural checks, and actual host
  execution clearly distinguished.
- Prefer small end-to-end experiments and existing compiler/linker components.
  Record consequential architecture choices under `docs/decisions/`.
- Pin upstream inputs for experiments. Keep generated artifacts in `artifacts/`.
- Add behavioral tests when implementing real behavior. Avoid tests that merely
  assert placeholder strings or restate the implementation.
- Run `just check` (or the equivalent Cargo commands in the README) before
  committing Rust changes.
- Keep failed controls and exact hashes in experiment reports. Do not claim a
  platform is supported from compilation or emulation alone.
- Keep unrelated changes intact and credentials out of Git and evidence.
- Treat tracked files, commit messages, PRs, issues, and uploaded evidence as
  public. Follow `docs/public-evidence.md` before publishing.
- Identify deployed machines with opaque public IDs such as `host-001`; never
  publish real hostnames, hostname-derived IDs, IP/MAC addresses, private DNS
  names, network/site names, usernames, home paths, or device serial numbers.
  Keep the identity mapping and operational configuration in private storage
  outside tracked files. Use reserved example names or loopback in examples.
- Publish reviewed evidence extracts, not raw logs, inventories, screenshots,
  credentials, token hashes, or deployment directories. Inspect free text and
  decoded stdout/stderr bytes as well as structured identity fields.
- Keep raw evidence immutable. Label substituted public IDs and source hashes
  explicitly; a sanitized extract is not the original hashed report. Before
  committing, review all staged content for private details, including embedded
  job/baseline IDs, filenames, links, and generated metadata.
