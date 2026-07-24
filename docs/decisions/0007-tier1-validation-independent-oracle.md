# 7. Tier-1 validation against an independent oracle (Fox-IT `dissect.util`)

Date: 2026-07-24
Status: Accepted

## Context

A clean-room codec that *produces a value* and could be cross-checked by an
independent oracle is precisely the case where a self-authored round-trip is
forbidden as the sole validation — the "LZNT1 trap": a wrong decoder and a
fixture hand-encoded to the same bug agree and ship green
(`ronin-issen/CLAUDE.core.md` → Evidence-Based Rigor / Doer-Checker). Correctness
must be established by something that does not depend on the code under test.

## Decision

Validate two orthogonal ways against **real Windows-produced data**
(`docs/validation.md`):

1. **Microsoft's own compressor as the round-trip oracle.** The primary test
   vector `tests/data/am_delta.xhuff` is the `MAM` payload of a real Win10
   prefetch file (`AM_DELTA.EXE-78CA83B0.pf`, Stolen Szechuan Sauce Case 001)
   with the 8-byte wrapper stripped — data Windows itself compressed. A faithful
   decoder must inflate it to exactly the container's declared 6948 bytes with the
   expected internal structure (valid `SCCA` header at the documented offset).
2. **An independent decompressor, byte-for-byte.** The same stream decoded by
   Fox-IT's independently-authored [`dissect.util`](https://github.com/fox-it/dissect.util)
   `lzxpress_huffman` — a separate [MS-XCA] implementation — compared by
   **SHA-256**; identical (`docs/validation.md`, with a reproducible cross-check
   script).

The in-crate tests assert full-output equality on two committed real vectors
(`am_delta`, 6948 bytes; `audiodg`, 35954 bytes, which exercises the
extended match-length ladder), plus the crafted malformed/EOF-padding paths
(`src/lib.rs` tests). The extended ladder is additionally confirmed on a 292 KB
real prefetch (`SEARCHHOST.EXE`) verified locally rather than committed, because
its redistribution status is unknown (`docs/validation.md`; Test-Data-Provenance
standard). Every artifact carries a provenance entry in `tests/data/README.md`.

## Consequences

- Correctness rests on Tier-1 evidence (real-world data + an independent oracle),
  not a self-consistent round-trip — the LZNT1 trap is structurally avoided.
- The committed vectors are small and clearly-sourced (public DFIR corpus), so CI
  is deterministic from committed bytes; the large third-party artifact stays
  gitignored and locally verified, per the provenance standard.
- Fuzzing ([ADR 0004](0004-panic-free-fuzzed-decoding.md)) covers *robustness*
  (never panic on hostile input); this ADR covers *correctness* (right bytes on
  real input) — the two are complementary, neither substitutes for the other.

[MS-XCA]: https://learn.microsoft.com/en-us/openspecs/windows_protocols/ms-xca/
