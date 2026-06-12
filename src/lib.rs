//! Pure-Rust decompressor for **Microsoft Xpress-Huffman**
//! (`LZXPRESS_HUFFMAN`, `COMPRESSION_FORMAT_XPRESS_HUFF` = 4), specified in
//! [MS-XCA] §2.2.4.
//!
//! Xpress-Huffman is the LZ77+Huffman codec that modern Windows uses pervasively:
//! Win8.1+ **prefetch** (`MAM` wrapper), **`hiberfil.sys`**, **SMB3** compression,
//! **registry-hive** compression, and Windows Update payloads. Unlike *plain*
//! `LZXpress` (`COMPRESSION_FORMAT_XPRESS` = 3, the `LZNT`-style format the existing
//! `rust-lzxpress` / `xpress_rs` crates implement), this is the Huffman-coded
//! variant.
//!
//! This crate is **cross-platform** — it does not call the Windows
//! `RtlDecompressBufferEx` API, so it decompresses Windows artifacts on Linux and
//! macOS just as well as on Windows. It is **panic-free**: every length/offset
//! field read from the (untrusted) input is bounds-checked, never indexed blindly.
//!
//! ```
//! # fn main() -> Result<(), xpress_huffman::Error> {
//! # let compressed: &[u8] = &[];
//! # let known_output_size = 0;
//! # if !compressed.is_empty() {
//! let plain = xpress_huffman::decompress(compressed, known_output_size)?;
//! # let _ = plain;
//! # }
//! # Ok(())
//! # }
//! ```
//!
//! Reimplemented clean-room from the [MS-XCA] algorithm (structure cross-checked
//! against Fox-IT's `dissect.util`; no code copied).
//! Spec: <https://learn.microsoft.com/en-us/openspecs/windows_protocols/ms-xca/>.

#![forbid(unsafe_code)]
#![cfg_attr(not(any(feature = "std", test)), no_std)]

extern crate alloc;
use alloc::vec;
use alloc::vec::Vec;

/// Errors that can arise while decompressing an Xpress-Huffman stream.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Error {
    /// A 64 KiB block's 256-byte Huffman code-length table was truncated.
    TruncatedTable,
    /// A back-reference pointed before the start of the output (corrupt stream).
    BadMatchOffset,
}

impl core::fmt::Display for Error {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Error::TruncatedTable => f.write_str("xpress-huffman: truncated Huffman table"),
            Error::BadMatchOffset => {
                f.write_str("xpress-huffman: match offset before output start")
            }
        }
    }
}

#[cfg(feature = "std")]
impl std::error::Error for Error {}

/// Per-block Huffman tree size: 512 symbols × 4-bit code lengths = 256 bytes.
const TABLE_LEN: usize = 256;
/// Each block decodes at most 64 KiB before a fresh table is read.
const BLOCK_SIZE: usize = 1 << 16;

/// Decompress an Xpress-Huffman stream into `decompressed_size` bytes.
///
/// `decompressed_size` is the output length recorded by the container that wraps
/// the stream (e.g. the 4-byte size after a prefetch `MAM\x04` signature, or the
/// uncompressed size in an SMB3 / hibernation header). Decoding stops once that
/// many bytes have been produced or the input is exhausted, whichever comes
/// first; the returned `Vec` is exactly the bytes decoded.
pub fn decompress(compressed: &[u8], decompressed_size: usize) -> Result<Vec<u8>, Error> {
    let mut dst: Vec<u8> = Vec::with_capacity(decompressed_size);
    let mut bs = BitStream::new(compressed);

    while bs.pos < compressed.len() && dst.len() < decompressed_size {
        let table = compressed
            .get(bs.pos..bs.pos + TABLE_LEN)
            .ok_or(Error::TruncatedTable)?;
        let tree = build_tree(table);
        bs.pos += TABLE_LEN;
        bs.init();

        let mut produced_in_block = 0usize;
        while produced_in_block < BLOCK_SIZE
            && bs.pos < compressed.len()
            && dst.len() < decompressed_size
        {
            let symbol = bs.decode(&tree);
            if symbol < 256 {
                dst.push(symbol as u8);
                produced_in_block += 1;
                continue;
            }
            let symbol = symbol - 256;
            let offset_bits = u32::from(symbol >> 4);
            let offset = (1usize << offset_bits) + bs.lookup(offset_bits) as usize;
            let length = bs.match_length(u32::from(symbol & 0x0F));
            bs.skip(offset_bits as i32);

            if offset == 0 || offset > dst.len() {
                return Err(Error::BadMatchOffset);
            }
            let mut remaining = length as usize;
            while remaining > 0 {
                let from = dst.len() - offset;
                let n = remaining.min(offset);
                for k in 0..n {
                    let b = dst[from + k];
                    dst.push(b);
                }
                remaining -= n;
            }
            produced_in_block += length as usize;
        }
    }
    Ok(dst)
}

/// One Huffman tree node (index-based; no per-child allocation).
#[derive(Clone, Copy)]
struct Node {
    children: [usize; 2],
    is_leaf: bool,
    symbol: u16,
}
const NONE: usize = usize::MAX;

/// Build the per-block Huffman decode tree from its 256-byte code-length table
/// (512 symbols, 4 bits each: byte `k` holds symbol `2k` in the low nibble,
/// `2k+1` in the high nibble). Canonical-code assignment per [MS-XCA].
fn build_tree(buf: &[u8]) -> Vec<Node> {
    let mut nodes = vec![
        Node {
            children: [NONE, NONE],
            is_leaf: false,
            symbol: 0,
        };
        1024
    ];

    let mut symbols: Vec<(u8, u16)> = Vec::with_capacity(512);
    for (i, &c) in buf.iter().enumerate().take(TABLE_LEN) {
        symbols.push((c & 0x0F, (i * 2) as u16));
        symbols.push((c >> 4, (i * 2 + 1) as u16));
    }
    symbols.sort_unstable();

    let start = symbols.iter().take_while(|(len, _)| *len == 0).count();

    let mut mask: u32 = 0;
    let mut bits: u32 = 1;
    let mut tree_index = 1usize;

    for &(length, symbol) in symbols.iter().take(512).skip(start) {
        let length = u32::from(length);
        {
            let node = &mut nodes[tree_index];
            node.symbol = symbol;
            node.is_leaf = true;
        }
        mask = mask.wrapping_shl(length.wrapping_sub(bits));
        bits = length;
        tree_index = add_leaf(&mut nodes, tree_index, mask, bits);
        mask = mask.wrapping_add(1);
    }
    nodes
}

/// Splice leaf node `idx` into the tree along the path described by `mask`/`bits`,
/// creating internal nodes as needed. Returns the next free node index.
fn add_leaf(nodes: &mut [Node], idx: usize, mask: u32, bits: u32) -> usize {
    let mut cur = 0usize;
    let mut i = idx + 1;
    let mut bits = bits;
    while bits > 1 {
        bits -= 1;
        let childidx = ((mask >> bits) & 1) as usize;
        if nodes[cur].children[childidx] == NONE {
            nodes[cur].children[childidx] = i;
            nodes[i].is_leaf = false;
            i += 1;
        }
        cur = nodes[cur].children[childidx];
    }
    nodes[cur].children[(mask & 1) as usize] = idx;
    i
}

/// Bit reader over the compressed stream: a 32-bit window refilled 16 bits at a
/// time from little-endian source words, with a byte cursor SHARED with the
/// extended-length / extra-offset byte reads (they interleave, per [MS-XCA]).
struct BitStream<'a> {
    data: &'a [u8],
    pos: usize,
    mask: u32,
    bits: i32,
}

impl<'a> BitStream<'a> {
    fn new(data: &'a [u8]) -> Self {
        Self {
            data,
            pos: 0,
            mask: 0,
            bits: 0,
        }
    }

    /// Read one 16-bit little-endian word; at EOF a lone trailing byte becomes
    /// the high byte. Advances the cursor by the bytes actually available (≤ 2).
    fn read16(&mut self) -> u32 {
        let avail = self.data.len().saturating_sub(self.pos);
        let v = match avail {
            0 => 0,
            1 => u32::from(self.data[self.pos]) << 8,
            _ => u32::from(u16::from_le_bytes([
                self.data[self.pos],
                self.data[self.pos + 1],
            ])),
        };
        self.pos += avail.min(2);
        v
    }

    fn init(&mut self) {
        self.mask = (self.read16() << 16).wrapping_add(self.read16());
        self.bits = 32;
    }

    fn lookup(&self, n: u32) -> u32 {
        if n == 0 {
            0
        } else {
            self.mask >> (32 - n)
        }
    }

    fn skip(&mut self, n: i32) {
        self.mask = self.mask.wrapping_shl(n as u32);
        self.bits -= n;
        if self.bits < 16 {
            self.mask = self.mask.wrapping_add(self.read16() << (16 - self.bits));
            self.bits += 16;
        }
    }

    fn read_byte(&mut self) -> u8 {
        let b = self.data.get(self.pos).copied().unwrap_or(0);
        self.pos += 1;
        b
    }

    /// Decode an LZ77 match length from its 4-bit symbol nibble ([MS-XCA]): a
    /// nibble of 15 escalates to a trailing length byte, and a byte of 255 (i.e.
    /// length 270) escalates again to a trailing 16-bit length word. The returned
    /// value already includes the +3 minimum-match bias.
    fn match_length(&mut self, nibble: u32) -> u32 {
        let mut length = nibble;
        if length == 15 {
            length = u32::from(self.read_byte()) + 15;
            if length == 270 {
                length = self.read16();
            }
        }
        length + 3
    }

    /// Walk the Huffman tree one bit at a time to a leaf symbol.
    fn decode(&mut self, nodes: &[Node]) -> u16 {
        let mut node = 0usize;
        while !nodes[node].is_leaf {
            let bit = self.lookup(1) as usize;
            self.skip(1);
            let next = nodes[node].children[bit];
            if next == NONE {
                return 0; // cov:unreachable: a valid tree always reaches a leaf
            }
            node = next;
        }
        nodes[node].symbol
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use super::*;

    // A real Xpress-Huffman test vector: the compressed stream from a Win10
    // prefetch file (Stolen Szechuan Sauce, AM_DELTA.EXE) with its 8-byte MAM
    // wrapper stripped, plus the expected decompressed payload. The expected
    // bytes were independently confirmed byte-for-byte by Fox-IT's dissect.util
    // decompressor (see docs/validation.md). Provenance: tests/data/README.md.
    const COMPRESSED: &[u8] = include_bytes!("../tests/data/am_delta.xhuff");
    const EXPECTED: &[u8] = include_bytes!("../tests/data/am_delta.expected");
    // A larger real vector (35954 bytes) — exercises the extended match-length
    // path (length == 270 → a trailing 16-bit length word).
    const AUDIODG: &[u8] = include_bytes!("../tests/data/audiodg.xhuff");
    const AUDIODG_EXPECTED: &[u8] = include_bytes!("../tests/data/audiodg.expected");

    #[test]
    fn decompresses_real_xpress_huffman_vector() {
        let out = decompress(COMPRESSED, EXPECTED.len()).unwrap();
        assert_eq!(out.len(), EXPECTED.len());
        assert_eq!(out, EXPECTED);
    }

    #[test]
    fn decompresses_larger_real_vector() {
        let out = decompress(AUDIODG, AUDIODG_EXPECTED.len()).unwrap();
        assert_eq!(out, AUDIODG_EXPECTED);
    }

    /// Build a one-symbol Huffman table whose first decoded symbol is the match
    /// symbol 256 (code length 1, code `0`). Decoding it before any literal has
    /// been emitted yields a match with offset 1 into an empty output.
    fn table_first_symbol_is_match() -> [u8; TABLE_LEN] {
        let mut t = [0u8; TABLE_LEN];
        // symbol 256 lives in byte 128's low nibble; length 1.
        t[128] = 0x01;
        t
    }

    #[test]
    fn match_before_any_output_errors() {
        // table + 4 init bytes + 2 padding bytes (so the decode loop runs): the
        // first decoded symbol is 256 → a match with offset 1, but the output is
        // empty → BadMatchOffset (also exercises the lookup(0) zero-bits path).
        let mut input = table_first_symbol_is_match().to_vec();
        input.extend_from_slice(&[0, 0, 0, 0, 0, 0]);
        assert_eq!(decompress(&input, 100), Err(Error::BadMatchOffset));
    }

    #[test]
    fn handles_init_at_exact_eof() {
        // Only 2 bytes after the table: init's second 16-bit read sees 0 bytes
        // left (EOF → 0). The stream is then exhausted, so the result is empty.
        let mut input = table_first_symbol_is_match().to_vec();
        input.extend_from_slice(&[0, 0]);
        assert_eq!(decompress(&input, 100), Ok(Vec::new()));
    }

    #[test]
    fn handles_init_with_one_trailing_byte() {
        // 3 bytes after the table: init's second 16-bit read sees a single
        // trailing byte (EOF padding → it becomes the high byte).
        let mut input = table_first_symbol_is_match().to_vec();
        input.extend_from_slice(&[0, 0, 0]);
        assert_eq!(decompress(&input, 100), Ok(Vec::new()));
    }

    #[test]
    fn stops_at_requested_size() {
        // Asking for fewer bytes than the stream holds returns exactly that many.
        let out = decompress(COMPRESSED, 16).unwrap();
        assert_eq!(out.len(), 16);
        assert_eq!(out, &EXPECTED[..16]);
    }

    #[test]
    fn empty_input_yields_empty() {
        assert_eq!(decompress(&[], 0).unwrap(), Vec::<u8>::new());
        assert_eq!(decompress(&[], 100).unwrap(), Vec::<u8>::new());
    }

    #[test]
    fn truncated_table_errors() {
        // Fewer than 256 bytes of table → TruncatedTable, no panic.
        assert_eq!(decompress(&[0u8; 10], 100), Err(Error::TruncatedTable));
    }

    #[test]
    fn error_is_display() {
        assert!(!format!("{}", Error::TruncatedTable).is_empty());
        assert!(!format!("{}", Error::BadMatchOffset).is_empty());
    }

    // The match-length escalation ladder ([MS-XCA]), unit-tested directly: a full
    // 273+ byte run (nibble 15 → byte 255 → trailing 16-bit word) is too rare to
    // appear in the small real vectors, so exercise the decision in isolation.
    #[test]
    fn match_length_short() {
        // nibble < 15: no escalation, just the +3 bias.
        let mut bs = BitStream::new(&[]);
        assert_eq!(bs.match_length(7), 10);
    }

    #[test]
    fn match_length_one_byte_extension() {
        // nibble 15, extension byte 10 (!= 255): length = 10 + 15 + 3.
        let mut bs = BitStream::new(&[10]);
        assert_eq!(bs.match_length(15), 28);
    }

    #[test]
    fn match_length_max_run_reads_16bit() {
        // nibble 15, extension byte 255 → 270 → trailing u16 (LE 0x1234) is the
        // real length. This is the line a 273+ byte run reaches.
        let mut bs = BitStream::new(&[255, 0x34, 0x12]);
        assert_eq!(bs.match_length(15), 0x1234 + 3);
    }
}
