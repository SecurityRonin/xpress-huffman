# 4. Panic-free, fuzzed decoding of untrusted streams

Date: 2026-07-24
Status: Accepted

## Context

Every byte this crate parses is attacker-controllable: the Huffman code-length
table, the bit stream, every match offset and length come from an untrusted
compressed blob (prefetch `MAM` payloads, `hiberfil.sys`, SMB3 frames, registry
hives). A length field that lies, a truncated table, or a back-reference pointing
before the output start must never crash the caller or — worse — read out of
bounds and emit silently wrong plaintext. The fleet's Paranoid Gatekeeper
standard (`ronin-issen/CLAUDE.md`) requires the static panic-free lint posture
*plus* a fuzz target per parsed structure for exactly this class of crate.

## Decision

Enforce robustness statically and dynamically.

**Static** (`Cargo.toml` `[lints.clippy]`): `unwrap_used` and `expect_used` are
`deny` in production; `correctness` and `suspicious` are `deny`. Tests are
exempted via `clippy.toml` (`allow-unwrap-in-tests`), not scattered `#[allow]`s.
Every field read from the stream is bounds-checked before use:

- The 256-byte table is fetched with `compressed.get(pos..pos+TABLE_LEN)` →
  `Error::TruncatedTable` on a short read (`src/lib.rs`, `decompress`).
- A match offset of 0 or one exceeding the bytes produced so far →
  `Error::BadMatchOffset` before any copy (`src/lib.rs`).
- The bit reader (`read16`, `read_byte`) returns 0 at EOF via `saturating_sub` /
  `slice::get(...).unwrap_or(0)` instead of indexing past the end.

**Dynamic** (`fuzz/fuzz_targets/decompress.rs`, commit `513220b`): a
`cargo-fuzz` / libFuzzer target feeds `(u32, &[u8])` — a caller-controlled size
hint and arbitrary bytes — asserting the invariant *every input yields `Ok` or a
typed `Err`, never a panic/abort/over-allocation*. The size hint is capped at
1 MiB in the harness so libFuzzer's RSS limit is never the thing under test (the
allocation bound is the caller's contract — see
[ADR 0006](0006-decompressed-size-supplied-out-of-band.md)). `ci.yml` runs
`cargo +nightly fuzz check` on every push; `fuzz.yml` runs a bounded campaign
weekly.

## Consequences

- Malformed evidence degrades to a typed `Error`, never a crash or an
  out-of-bounds read; the two error variants (`TruncatedTable`, `BadMatchOffset`)
  name what was rejected.
- The static lints occasionally demand more verbose bounds-checked code than a
  quick `unwrap` would; that is the intended cost.
- The static/dynamic pairing is the fleet's "input-fuzzed" (measured, tier-1) +
  "panic-free by lint" (static) posture — the README leads with fuzzing evidence
  and qualifies the panic-free claim accordingly (`README.md`,
  `docs/validation.md`), never a bare "panic-free" absolute.
- One genuinely-unreachable defensive arm (`decode`'s `NONE`-child guard) is kept
  and annotated `// cov:unreachable` rather than deleted, per the coverage-gate
  standard (commit `365cdd2`).
