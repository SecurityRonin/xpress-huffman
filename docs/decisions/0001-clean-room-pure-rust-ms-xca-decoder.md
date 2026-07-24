# 1. Clean-room pure-Rust decoder for [MS-XCA] Xpress-Huffman

Date: 2026-07-24
Status: Accepted

## Context

Modern Windows compresses a large share of forensically interesting data with
**Xpress-Huffman** (`COMPRESSION_FORMAT_XPRESS_HUFF`, value 4; `LZXPRESS_HUFFMAN`),
specified in [MS-XCA] §2.2.4: Win8.1+ **prefetch** (the `MAM` wrapper),
**`hiberfil.sys`**, **SMB3** transport compression, **registry-hive** compression,
and Windows Update payloads (`src/lib.rs` module docs; `README.md`). Off-Windows
the usual route is to shell out to `RtlDecompressBufferEx`, which exists only on
Windows — useless to a Linux/macOS analysis pipeline.

The Rust ecosystem was surveyed before writing a line (Research-First discipline,
`ronin-issen/CLAUDE.core.md`). The existing crates — `rust-lzxpress`, `xpress_rs` —
implement only *plain* LZXpress (`COMPRESSION_FORMAT_XPRESS`, value 3; the
`LZNT`-style LZ77 format), **not** the Huffman-coded variant (value 4) that the
artifacts above actually use (`README.md` comparison table). So no reusable crate
covered format 4. This is the exact case the fleet constitution names as the
justified exception to "reuse before build": *"a format-specific codec with no
ecosystem implementation (e.g. MS Xpress-Huffman / [MS-XCA])"*
(`CLAUDE.core.md` → Research-First; Rust Lint Posture / unsafe law).

## Decision

Implement the Xpress-Huffman decompressor **clean-room** from the [MS-XCA] §2.2.4
algorithm, in pure Rust with no Windows API and no FFI. Structure was
cross-checked against Fox-IT's independently-authored `dissect.util`
implementation for correctness, but no code was copied (`src/lib.rs` header;
initial commit `63f5732`). The spec dictates the wire structure, which the code
follows literally rather than by choice:

- **Per-64 KiB block** decoding: each block begins with a fresh Huffman
  code-length table and decodes at most `BLOCK_SIZE = 1 << 16` bytes
  (`src/lib.rs`).
- **256-byte code-length table** = 512 symbols × 4-bit lengths (`TABLE_LEN = 256`);
  byte `k` holds symbol `2k` in the low nibble and `2k+1` in the high nibble;
  canonical-code assignment per the spec (`build_tree`).
- **Little-endian** bit stream: a 32-bit window refilled 16 bits at a time from
  LE source words (`BitStream::read16` uses `u16::from_le_bytes`).
- **LZ77 match-length escalation ladder**: 4-bit nibble → trailing byte at 15 →
  trailing 16-bit LE word at 270, with the +3 minimum-match bias
  (`BitStream::match_length`).

## Consequences

- Windows-produced artifacts decompress identically on Linux, macOS, and Windows;
  no `RtlDecompressBufferEx`, no platform gate.
- The fleet gains the format-4 decoder its prefetch/hiberfil/SMB3/registry
  pipelines need; `prefetch-forensic` builds on this crate (`README.md`).
- A clean-room codec that emits a value and can be cross-checked by an
  independent oracle must be validated at Tier-1, not by a self-authored
  round-trip (the "LZNT1 trap"). That validation is a separate decision
  ([ADR 0007](0007-tier1-validation-independent-oracle.md)).
- Because the byte/table/block layout is spec-mandated, the risk is an inverted
  bit-split or wrong offset shipping green — mitigated by the independent-oracle
  cross-check on real artifacts and by fuzzing
  ([ADR 0004](0004-panic-free-fuzzed-decoding.md)).

[MS-XCA]: https://learn.microsoft.com/en-us/openspecs/windows_protocols/ms-xca/
