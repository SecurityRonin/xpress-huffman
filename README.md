# xpress-huffman

[![Crates.io](https://img.shields.io/crates/v/xpress-huffman.svg)](https://crates.io/crates/xpress-huffman)
[![Docs.rs](https://docs.rs/xpress-huffman/badge.svg)](https://docs.rs/xpress-huffman)
[![Rust 1.85+](https://img.shields.io/badge/rust-1.85%2B-orange.svg)](https://www.rust-lang.org)
[![License: Apache-2.0](https://img.shields.io/badge/license-Apache--2.0-blue.svg)](LICENSE)
[![Sponsor](https://img.shields.io/badge/sponsor-h4x0r-ea4aaa.svg)](https://github.com/sponsors/h4x0r)

[![CI](https://github.com/SecurityRonin/xpress-huffman/actions/workflows/ci.yml/badge.svg)](https://github.com/SecurityRonin/xpress-huffman/actions/workflows/ci.yml)
[![unsafe forbidden](https://img.shields.io/badge/unsafe-forbidden-success.svg)](https://github.com/rust-secure-code/safety-dance/)
[![Security advisories](https://img.shields.io/badge/security-cargo--deny-success.svg)](deny.toml)

**Decompress Microsoft Xpress-Huffman ([MS-XCA] §2.2.4) anywhere — the codec behind Win10+ prefetch, `hiberfil.sys`, SMB3 and registry-hive compression — in pure, panic-free Rust with no Windows API.**

```rust
// `compressed` is an LZXPRESS_HUFFMAN stream (e.g. a prefetch MAM payload after
// the 8-byte header); `size` is the decompressed length the container records.
let plain = xpress_huffman::decompress(compressed, size)?;
```

That's the whole surface: one function, no setup, `#![no_std]`-compatible.

## Why this crate

Modern Windows compresses a *lot* with Xpress-Huffman (`COMPRESSION_FORMAT_XPRESS_HUFF`, value 4): Win8.1+ **Prefetch** (the `MAM` wrapper), **`hiberfil.sys`**, **SMB3** transport compression, **registry hive** compression, and Windows Update payloads. Decoding it off-Windows usually means shelling out to `RtlDecompressBufferEx` — which only exists on Windows.

| | Algorithm | Decodes off-Windows? |
|---|---|---|
| **`xpress-huffman` (this crate)** | Xpress-**Huffman** ([MS-XCA] §2.2.4) | ✅ pure Rust, any platform |
| `rust-lzxpress`, `xpress_rs` | plain LZXpress (`COMPRESSION_FORMAT_XPRESS` = 3) | ✅ — but *not the Huffman variant* |
| `RtlDecompressBufferEx` (Windows API) | both | ❌ Windows only |

The existing Rust crates implement *plain* LZXpress (the `LZNT`-style LZ77 format, value 3). This crate implements the **Huffman-coded** variant (value 4) that the artifacts above actually use.

## Trust, but verify

- **`#![forbid(unsafe_code)]`**, no `unwrap`/`expect`/panic in production paths — every length and offset read from the (untrusted) input is bounds-checked. A corrupt stream yields an `Err`, never a panic or out-of-bounds read.
- **`#![no_std]`** (uses `alloc`); enable the `std` feature for a `std::error::Error` impl.
- **Validated against an independent decompressor.** The decoder is a clean-room implementation of the [MS-XCA] algorithm; its output is confirmed **byte-for-byte** against Fox-IT's independent `dissect.util` decompressor on real Windows artifacts. See [`docs/validation.md`](docs/validation.md).

## Install

```toml
[dependencies]
xpress-huffman = "0.1"
```

## Decoding a Windows prefetch payload

```rust
// A Win8.1+ prefetch file: `MAM\x04` + u32 decompressed size + the stream.
let size = u32::from_le_bytes(mam[4..8].try_into().unwrap()) as usize;
let scca = xpress_huffman::decompress(&mam[8..], size)?;
assert_eq!(&scca[4..8], b"SCCA");
```

(For full prefetch parsing — run counts, last-run times, loaded files — see the
[`prefetch-forensic`](https://github.com/SecurityRonin/prefetch-forensic) crate, which builds on this one.)

[MS-XCA]: https://learn.microsoft.com/en-us/openspecs/windows_protocols/ms-xca/

---

[Privacy Policy](https://securityronin.github.io/xpress-huffman/privacy/) · [Terms of Service](https://securityronin.github.io/xpress-huffman/terms/) · © 2026 Security Ronin Ltd
