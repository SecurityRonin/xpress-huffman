# 5. `#![no_std]` + alloc; one-function API; opt-in `std` for the `Error` impl

Date: 2026-07-24
Status: Accepted

## Context

This crate is a leaf codec meant to be linked by many consumers, some of which
may themselves target constrained or `no_std`-adjacent environments. Its only
runtime need is heap allocation (a growable output `Vec` and the decode tree).
Pulling in `std` unconditionally would foreclose those consumers for no gain, and
a wide API surface (streaming readers, writer traits, configuration structs)
would be surface to maintain, test, and get wrong — none of it required to turn
one compressed blob into its plaintext (YAGNI / Scope Fidelity,
`CLAUDE.core.md`).

## Decision

- **`#![no_std]` + `extern crate alloc`** (`src/lib.rs`): the crate uses only
  `alloc::vec::Vec`; `std` is not required. `no-std` is declared in `categories`
  (`Cargo.toml`).
- **One public function**: `decompress(compressed: &[u8], decompressed_size:
  usize) -> Result<Vec<u8>, Error>`. That is the whole surface (`README.md`: "one
  function, no setup"). Decode-only; there is no compressor (not needed by any
  consumer today).
- **`std` is an opt-in feature, off by default** (`Cargo.toml` `[features]`),
  gating only `impl std::error::Error for Error`. `no_std` builds still get a
  `core::fmt::Display` `Error` (`src/lib.rs`), so diagnostics are never lost — the
  feature adds trait interop, not capability.

## Consequences

- Consumers get a portable, dependency-free codec that compiles in `no_std`
  contexts; a `std` consumer flips one feature to get `std::error::Error`
  interop.
- The minimal API is Secure-by-Default: the zero-config path
  (`decompress(bytes, size)`) is the safe and only path; there are no
  "unsafe-but-fast" escape hatches to misuse.
- Adding compression later, or a streaming variant, is an additive,
  non-breaking change on this crate ([ADR 0002](0002-single-pure-codec-crate-no-core-forensic-split.md)),
  not a reason to widen the surface pre-emptively now.
