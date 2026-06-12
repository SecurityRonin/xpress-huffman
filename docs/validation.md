# Validation

## Summary

`xpress-huffman`'s decoder is a clean-room implementation of the Microsoft
Xpress-Huffman algorithm ([MS-XCA] §2.2.4). Correctness is established two ways
against real Windows-produced data — neither of which relies on the code under
test:

1. **Microsoft's own compressor as the round-trip oracle.** The test vector is a
   real Win10 Prefetch payload, which Windows itself compressed with
   Xpress-Huffman. A faithful decoder must reproduce a byte-exact payload of the
   container's declared length. Our decoder inflates it to exactly the declared
   size and the output carries the expected internal structure (a valid `SCCA`
   header at the documented offset).

2. **An independent decompressor, byte-for-byte.** The same compressed stream was
   decompressed with Fox-IT's [`dissect.util`](https://github.com/fox-it/dissect.util)
   `lzxpress_huffman` — a separate, independently-authored [MS-XCA] implementation
   — and the two outputs were compared by **SHA-256**. They are identical.

## Test vector

| Field | Value |
|---|---|
| Source | `AM_DELTA.EXE-78CA83B0.pf` (Stolen Szechuan Sauce, Case 001 Desktop image) |
| Compressed stream | `tests/data/am_delta.xhuff` — the prefetch `MAM` payload with the 8-byte wrapper stripped (1858 bytes) |
| Expected output | `tests/data/am_delta.expected` — 6948 bytes |
| Independent oracle | `dissect.util.compression.lzxpress_huffman` |
| Result | **byte-identical** (SHA-256 match) |

The in-crate test `decompresses_real_xpress_huffman_vector` asserts the full
6948-byte output equals `am_delta.expected`; `decompresses_larger_real_vector`
does the same for a 35954-byte vector (`audiodg`); `stops_at_requested_size`,
`empty_input_yields_empty`, `truncated_table_errors`, and the crafted
`match_before_any_output_errors` / `handles_init_*` tests cover the
bounded-output, malformed-input, and EOF-padding paths.

### Large real artifact (292 KB, max-length matches)

The committed vectors are small, so the extended match-length ladder
(`match_length`: a 273+ byte run encoded as nibble 15 → byte 255 → trailing
16-bit length word) is unit-tested directly. It is additionally confirmed on a
real **292 KB** Win10 prefetch (`SEARCHHOST.EXE`, which contains long matches):
our decoder's output is **byte-identical** (SHA-256
`1ea0eb103e8935a664eb513edbd6551ad779d9b60803ad39ff53b4afb59d754e`) to
`dissect.util`'s. That artifact is from a third-party corpus of unknown
redistribution status, so it is verified locally rather than committed.

## Reproducing the independent cross-check

```bash
python3 -m venv /tmp/xh && /tmp/xh/bin/pip install dissect.util
/tmp/xh/bin/python - <<'PY'
import io, hashlib
from dissect.util.compression import lzxpress_huffman
comp = open("tests/data/am_delta.xhuff", "rb").read()
exp  = open("tests/data/am_delta.expected", "rb").read()
out  = lzxpress_huffman.decompress(io.BytesIO(comp))[:len(exp)]
print("dissect == expected:", out == exp,
      "| sha256:", hashlib.sha256(out).hexdigest())
PY
```

[MS-XCA]: https://learn.microsoft.com/en-us/openspecs/windows_protocols/ms-xca/
