# 3. `#![forbid(unsafe_code)]` — full forbid, not deny + bounded allow

Date: 2026-07-24
Status: Accepted

## Context

The fleet's unsafe law (`ronin-issen/CLAUDE.core.md` → "unsafe Is an Avoidable
Cost-Benefit Exception") sets `forbid(unsafe)` as the default *and the goal* — a
provable, badge-able "zero places a crafted input can corrupt memory." It permits
a downgrade to `unsafe_code = "deny"` + a bounded per-site `#[allow]` **only when
a real benefit justifies it** — the canonical case being an `mmap` scanner
(`ewf`, `memory-forensic`) that needs one bounded `unsafe` block for performance.

This crate is a **pure in-memory codec**: it takes a `&[u8]` and produces a
`Vec<u8>` (`src/lib.rs`). There is no file mapping, no zero-copy scan over a
multi-GiB image, no FFI — so none of the benefits that justify a downgrade apply.
Its entire input is attacker-controllable (untrusted compressed streams from
prefetch, hiberfil, SMB3, registry hives).

## Decision

Set **`unsafe_code = "forbid"`** crate-wide (`Cargo.toml` `[lints.rust]`;
`#![forbid(unsafe_code)]` in `src/lib.rs`). No downgrade to `deny`, no per-site
allow. `rg 'unsafe'` over `src/` is empty by construction, and `forbid` cannot be
locally overridden, so the guarantee is airtight.

## Consequences

- The crate earns the honest **`unsafe forbidden`** README badge (`README.md`) —
  not the "`deny` + N bounded allows" wording the mmap crates must use.
- The index-based Huffman tree (`Node { children: [usize; 2] }`, `NONE =
  usize::MAX`) and the bit-window reads are written with bounds-checked indexing
  and `slice::get`, never raw pointers — the safe alternative that `forbid`
  requires and that a codec loses nothing by taking.
- Consistent with the fleet's "one bar, applied fleet-wide": a crate with no
  perf-critical unsafe benefit takes the strict end; only a demonstrated benefit
  (mmap) moves to `deny` + allow.
