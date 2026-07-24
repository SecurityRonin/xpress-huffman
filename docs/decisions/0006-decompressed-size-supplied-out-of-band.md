# 6. Decompressed size is supplied out-of-band by the caller

Date: 2026-07-24
Status: Accepted

## Context

The [MS-XCA] Xpress-Huffman bit stream carries no self-describing total output
length. In every real container the decompressed size is recorded *outside* the
stream: the 4-byte size after a prefetch `MAM\x04` signature, the uncompressed
size in an SMB3 `COMPRESSION_TRANSFORM_HEADER`, the hibernation slot header, etc.
(`src/lib.rs`, `decompress` docs; `README.md`). A decoder therefore has two
honest options: stop when the *stream* is exhausted, or stop at a *caller-declared
size*. Guessing an internal length from an untrusted stream would invite an
allocation bomb — a lying length field driving an unbounded `Vec`.

## Decision

`decompress` takes **`decompressed_size: usize` as an explicit second argument**,
sourced by the caller from the wrapping container (`src/lib.rs`). It is used two
ways: as the `Vec::with_capacity` hint, and as a hard stop — decoding halts once
`dst.len() >= decompressed_size` **or** the input is exhausted, whichever comes
first; the returned `Vec` is exactly the bytes decoded (`src/lib.rs`; tests
`stops_at_requested_size`, `empty_input_yields_empty`).

Because the size is the caller's, a hostile value is an allocation bomb the
**caller** controls, not a decoder defect — documented explicitly in the fuzz
harness, which therefore caps its own derived size at 1 MiB
(`fuzz/fuzz_targets/decompress.rs`).

## Consequences

- The decoder never trusts a length embedded in an untrusted stream; the only
  allocation bound is the value the caller passes, keeping the security boundary
  where the caller can reason about it (Secure-by-Design).
- Callers that only want a prefix (triage, header sniffing) pass a small size and
  get exactly that many bytes back without decoding the whole stream
  (`stops_at_requested_size`).
- The contract puts one obligation on the caller — supply the container's
  recorded size — which the container always has; there is no realistic case
  where the size is truly unknown, so no streaming/grow-forever mode is offered
  (YAGNI).
