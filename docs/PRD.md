# xpress-huffman — Purpose & Scope

*This is a **library-tier** intent doc: a concise Purpose & Scope, not a product
requirements document. `xpress-huffman` ships no binary an examiner runs — it is a
pure-computation codec that other crates link (fleet PRD/ADR standard,
`ronin-issen/CLAUDE.md`). Every current-state claim below is grounded in a
same-session read of `src/lib.rs`, `Cargo.toml`, `README.md`, and
`docs/validation.md` (2026-07-24). The load-bearing decisions live as ADRs
[0001](decisions/0001-clean-room-pure-rust-ms-xca-decoder.md)–[0008](decisions/0008-declared-msrv-below-pinned-dev-toolchain.md)
under [`docs/decisions/`](decisions/).*

## What it is

A pure-Rust decompressor for **Microsoft Xpress-Huffman**
(`LZXPRESS_HUFFMAN` / `COMPRESSION_FORMAT_XPRESS_HUFF` = 4), specified in
[MS-XCA] §2.2.4. One public function turns a compressed stream plus its
caller-known output length into the decompressed bytes:

```rust
let plain = xpress_huffman::decompress(compressed, decompressed_size)?;
```

That is the entire surface — no setup, no configuration, `#![no_std]` + `alloc`
([ADR 0005](decisions/0005-no-std-single-function-api.md)).

## Why it exists

Modern Windows compresses a large share of forensically interesting data with
Xpress-Huffman: Win8.1+ **prefetch** (the `MAM` wrapper), **`hiberfil.sys`**,
**SMB3** transport compression, **registry-hive** compression, and Windows Update
payloads. Off-Windows the usual route is `RtlDecompressBufferEx`, which only
exists on Windows. The existing Rust crates (`rust-lzxpress`, `xpress_rs`)
implement only *plain* LZXpress (format 3), not the Huffman-coded variant
(format 4) these artifacts use. This crate fills that gap with a clean-room
[MS-XCA] implementation that runs anywhere
([ADR 0001](decisions/0001-clean-room-pure-rust-ms-xca-decoder.md)).

## Who links it

Fleet parsers that must decode a format-4 stream before they can analyze the
plaintext — `prefetch-forensic` (the `MAM` payload), and future
registry-hive / `hiberfil.sys` / SMB3 consumers. It is a zero-fleet-dependency
leaf: everything depends *down* onto it; it depends on nothing
([ADR 0002](decisions/0002-single-pure-codec-crate-no-core-forensic-split.md)).
Being `no_std` + dependency-free, it is equally usable by third-party consumers
outside the fleet.

## Scope

- Decode a single Xpress-Huffman ([MS-XCA] §2.2.4) stream to a caller-declared
  output length, across the per-64 KiB-block Huffman + LZ77 structure the spec
  defines.
- Stop at the requested size or at input exhaustion, returning exactly the bytes
  decoded ([ADR 0006](decisions/0006-decompressed-size-supplied-out-of-band.md)).
- Reject malformed input (truncated table, back-reference before output start)
  with a typed `Error`, never a panic or out-of-bounds read
  ([ADR 0004](decisions/0004-panic-free-fuzzed-decoding.md)).
- Optional `std` feature adds `impl std::error::Error`; the default `no_std`
  build keeps a `Display` error.

## Non-goals

- **Compression.** Decode only; nothing in the fleet needs to *produce* format-4
  streams. Adding it later is additive on this crate, not a reason to build it now.
- **Container parsing.** Stripping the prefetch `MAM` wrapper, reading the SMB3
  header, or locating the `hiberfil.sys` compression slot is the *caller's* job;
  this crate receives the raw stream and its size. (The README shows the `MAM`
  unwrap as a caller convenience example, not a crate feature.)
- **Plain LZXpress (format 3).** Out of scope — served by `rust-lzxpress` /
  `xpress_rs`; this crate is the Huffman variant only.
- **A forensic-analysis (`-forensic`) companion.** A codec has no anomalies to
  grade; consumers analyze the decompressed plaintext
  ([ADR 0002](decisions/0002-single-pure-codec-crate-no-core-forensic-split.md)).

## Correctness & robustness

Correctness is Tier-1: the decoder's output is confirmed **byte-for-byte** (by
SHA-256) against Fox-IT's independent `dissect.util` decompressor on real Windows
prefetch artifacts, and against the containers' Microsoft-declared output sizes
([ADR 0007](decisions/0007-tier1-validation-independent-oracle.md);
[`docs/validation.md`](validation.md)). Robustness is enforced statically
(`#![forbid(unsafe_code)]`, `clippy::unwrap_used`/`expect_used` denied,
bounds-checked reads — ADRs
[0003](decisions/0003-forbid-unsafe-code.md)/[0004](decisions/0004-panic-free-fuzzed-decoding.md))
and dynamically (a `cargo-fuzz` target asserting *every input yields `Ok` or a
typed `Err`, never a panic*, run in CI and a weekly campaign).

[MS-XCA]: https://learn.microsoft.com/en-us/openspecs/windows_protocols/ms-xca/
