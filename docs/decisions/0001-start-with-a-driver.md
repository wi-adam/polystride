# ADR 0001: Start with a Rust build driver and packager

- Date: 2026-10-20
- Status: accepted as the initial research approach

## Context

The desired output is one identical executable across operating systems and
architectures. The first unknown is how a clean host launches that artifact.
Implementing object resolution and relocation does not answer that question.

## Decision

Begin with a small Rust workspace, reuse existing compilers and linkers, and
evaluate APE as a bootstrap candidate. Build only the orchestration and packaging
needed to run a complete small experiment. Select the final runtime, artifact
format, and payload strategy after the bootstrap evidence is available.

Retain custom linking as an option if an experiment identifies a concrete need.

## Consequences

- We can investigate feasibility without first implementing a full linker.
- The first working artifact may include upstream components.
- Upstream versions and runtime prerequisites must be explicit in every report.
- Initial experiments may fail the strict no-prerequisites goal; preserve those
  results and the original requirement.
- This decision does not claim that ordinary Rust applications are portable.
