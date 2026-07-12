//! Fuzz the Xpress-Huffman decompressor on arbitrary bytes.
//!
//! `decompress` walks per-64-KiB-block Huffman code-length tables, builds a
//! canonical decode tree, and replays LZ77 literals / back-references from a
//! bit stream — every field of which is attacker-controlled here. On any input
//! it must return `Ok` or a typed `Err` (`TruncatedTable` / `BadMatchOffset`),
//! never panic, abort, or over-allocate.
//!
//! `decompressed_size` is a caller-supplied capacity hint (`Vec::with_capacity`),
//! so a hostile value is an allocation bomb the *caller* controls, not a decoder
//! bug. The harness therefore derives it from the input and caps it at 1 MiB —
//! large enough to drive the multi-block loop (16 × 64 KiB) and the
//! stop-at-requested-size / overshoot paths, small enough that libFuzzer's RSS
//! limit is never the thing under test.
#![no_main]
use libfuzzer_sys::fuzz_target;

const SIZE_CAP: usize = 1 << 20;

fuzz_target!(|input: (u32, &[u8])| {
    let (requested, data) = input;
    let decompressed_size = requested as usize % (SIZE_CAP + 1);
    let _ = xpress_huffman::decompress(data, decompressed_size);
});
