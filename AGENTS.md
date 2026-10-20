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
