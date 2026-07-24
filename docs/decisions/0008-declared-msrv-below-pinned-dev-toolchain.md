# 8. Declared MSRV 1.85 sits below the pinned 1.96.0 dev toolchain

Date: 2026-07-24
Status: Accepted

## Context

The fleet toolchain policy (`ronin-issen/CLAUDE.core.md` → "Rust MSRV & Toolchain
Policy") separates two things that must not be conflated: the **dev toolchain**
(what everyone builds/fmt/clippy with) and the **declared MSRV** (`rust-version`,
a downstream-facing compatibility promise). A published library keeps a **low,
CI-verified MSRV** — raising it narrows the crates.io audience and is treated as a
near-breaking change. This crate is a published library leaf
([ADR 0002](0002-single-pure-codec-crate-no-core-forensic-split.md)), so the
policy applies.

## Decision

- **Pin the dev toolchain to the current fleet stable** in
  `rust-toolchain.toml` — `channel = "1.96.0"`, `components = ["clippy",
  "rustfmt"]` (commit `68bb0b1`, "pin toolchain to 1.96.0 — fleet toolchain
  policy"). One version across contributors + CI ends fmt/clippy drift.
- **Declare a lower `rust-version = "1.85"`** in `Cargo.toml` (README badge
  `Rust 1.85+`) as the downstream promise — deliberately below the dev pin, not
  equal to it, because a library must not force consumers onto the newest stable.

## Consequences

- Consumers can build against a Rust older than the fleet's dev pin; the promise
  is CI-verifiable and is a trust signal on crates.io.
- Raising 1.85 later is a deliberate, near-breaking bump requiring an explicit
  reason (a genuinely-needed newer-Rust feature), not a drive-by match to the dev
  toolchain.
- **Unrecovered rationale:** the specific choice of `1.85` (rather than the
  fleet's more common `1.75`/`1.80` library floor) is not explained in any commit
  message or config comment in the available history. Rationale reconstructed from
  structure; original intent not recovered in available history. The dev-pin ≠
  declared-MSRV *split* is the load-bearing, grounded decision; the exact floor
  value is recorded here as declared, not justified.
