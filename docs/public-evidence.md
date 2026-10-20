# Public evidence and host privacy

This repository is public. Apply these rules to documentation, examples,
manifests, commit messages, PRs, issues, and attachments.

## Host identity

Use a stable opaque public ID such as `host-001`. Assign it independently of a
machine's hostname, owner, address, physical location, or operational role.
Record the OS, architecture, and relevant baseline qualifications separately.
The public host registry is the [infrastructure matrix](validation.md#m05-infrastructure-qualification).

Keep the mapping from public IDs to operational identities in private storage
outside tracked files. An ignored local file is suitable if access is restricted
and it is never force-added or uploaded. Existing live credentials, immutable
jobs, receipts, and raw reports need not be renamed to publish a sanitized
extract. New deployments should use opaque IDs from the start.

Do not publish real hostnames or hostname-derived IDs, IP/MAC addresses, private
DNS names, network/site names, user or home-directory names, hardware serials,
or detailed inventories. A certificate, URL, screenshot, error, filename, or
job/baseline ID can expose the same information. Public project/upstream URLs
and relevant OS versions, CPU architecture, build inputs, and artifact hashes
are appropriate evidence. Keep hardware details to what the experiment needs.

Examples use synthetic IDs, reserved names such as `lab.example.invalid`, and
loopback addresses. They must not contain a working deployment endpoint.

## Evidence extracts

Keep original jobs and reports immutable in private storage. Publish only
reviewed fields needed to assess the result, retaining successful and failed
controls and clearly identifying any redaction or identity substitution.

The deployment manifests use these conventions:

- `public_host_id`, `public_job_id`, and `public_baseline_id` identify substituted
  public labels. They are not necessarily live protocol identifiers.
- `source_report_sha256` and `source_job_sha256` are digests of the original
  private evidence, not of the edited JSON extract or a reconstructed job.
- Artifact and agent hashes, byte counts, observed output, outcomes, and timing
  values remain unchanged unless an explicit redaction says otherwise.
- A redacted output stream omits `bytes`, sets `bytes_redacted: true`, explains
  why, and preserves `observed_size`, `truncated`, and a `source_bytes_sha256`
  digest of the original captured bytes. The digest does not cover an invented
  replacement string or bytes beyond the original capture limit.
- A `publication` entry describes the extract's identity and digest policy.
  Other manifests must document any different convention.

Inspect decoded stdout/stderr byte arrays, environment entries, free-text notes,
exceptions, paths, and metadata before publication. If an observation contains
private information, omit or redact it explicitly and retain the original
privately; do not fabricate a replacement observation or recompute a source hash
over edited evidence. Credentials, enrollment-token hashes, private keys, and
whole deployment directories are never public evidence.

Before committing, review the complete staged diff, parse changed manifests,
check links, and search tracked content for private identifiers and addresses.
Do not commit an audit denylist containing the private values it is meant to
detect. Avoid leaking them in the commit message or review description too.

## Previously published material

Editing the current branch does not remove information from earlier commits,
forks, caches, or existing downloads. Report any known historical exposure
accurately. A published-history rewrite requires explicit authorization because
it changes commit IDs and disrupts existing checkouts; it cannot erase copies
already obtained by others.
