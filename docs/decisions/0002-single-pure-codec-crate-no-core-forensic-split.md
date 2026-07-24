# 2. Single pure-codec crate — no core/forensic split; KNOWLEDGE-leaf role

Date: 2026-07-24
Status: Accepted

## Context

The fleet's default crate shape for a format is the two-crate
reader/analyzer split — `<x>-core` (reader) + `<x>-forensic` (analyzer) —
documented in `ronin-issen/CLAUDE.md` ("Crate-structure standard"). That pattern
exists so a *forensic auditor* can see raw byte layout, slack, and malformed
records that a happy-path reader normalizes away.

Xpress-Huffman is not a format an auditor inspects for anomalies; it is a
**compression codec** — bytes in, bytes out. There is no "anomaly" to grade, no
`AnomalyKind`, no `forensicnomicon::report::Finding` to emit. The consumers that
*do* audit — `prefetch-forensic`, registry/hive analyzers, SMB/hiberfil parsers —
sit above this crate and call it to obtain a decompressed stream, then do their
own analysis on the plaintext.

## Decision

Ship **one crate**, `xpress-huffman`, exposing a single decode entry point
(`Cargo.toml`; `src/lib.rs`). Do **not** apply the `-core`/`-forensic` split.
The crate is a **pure-computation codec leaf**, conceptually adjacent to the
KNOWLEDGE layer: it depends on nothing in the fleet and everything depends *down*
onto it.

The bare name `xpress-huffman` is taken (no third-party collision to route
around, so no `-core` suffix is needed for the import path, unlike the
collision-driven `bluetooth-forensic-core` / `zfs-forensic-core` cases in the
naming grammar). The name is self-describing on crates.io: it states the exact
[MS-XCA] variant (Huffman) that distinguishes it from the plain-LZXpress crates
(`README.md`).

## Consequences

- No `-forensic` companion, no `AnomalyKind`, no `report` dependency — correct
  for a codec, and it keeps the crate a zero-fleet-dependency leaf reusable by
  any consumer (in or out of the fleet).
- This is the fleet-scoped, repo-level ADR set for a single-crate repo (the
  PRD/ADR standard is repo-level, not per-crate).
- Should Xpress-Huffman *compression* ever be needed (it is decode-only today,
  [ADR 0005](0005-no-std-single-function-api.md)), it is an additive function on
  this same crate, not a new crate.
- Rationale reconstructed from structure and the fleet naming grammar; no commit
  explicitly debates the single-crate choice, but it is the only shape
  consistent with a pure codec under `CLAUDE.md`'s "pure-computation codecs" rule.
