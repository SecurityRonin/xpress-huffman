# Test data

Single repo-root `tests/data/` (fleet standard). These are committed (small,
public-provenance) Xpress-Huffman test vectors backing the in-crate tests.

See the fleet catalog: `issen/docs/corpus-catalog.md`.

#### am_delta.xhuff / am_delta.expected

- **What**: a real Microsoft Xpress-Huffman ([MS-XCA] §2.2.4) compressed stream
  and its decompressed output.
- **Source / Identity**: derived from `AM_DELTA.EXE-78CA83B0.pf`, a Windows 10
  Prefetch file from the **Stolen Szechuan Sauce** DFIR case (Case 001, Desktop
  image). `am_delta.xhuff` is that file's `MAM\x04` payload with the 8-byte
  wrapper (signature + size) stripped; `am_delta.expected` is the decompressed
  6948-byte `SCCA` payload.
- **Writeup**: https://thedfirreport.com/2020/11/30/stolen-szechuan-sauce/ ·
  dataset: https://github.com/dlcowen/TheStolenSzechuanSauce
- **Generator** (verbatim, from a checkout of the prefetch fixtures):
  ```sh
  python3 - <<'PY'
  import struct
  raw = open("AM_DELTA.EXE-78CA83B0.pf","rb").read()       # MAM-wrapped prefetch
  size = struct.unpack_from("<I", raw, 4)[0]               # 6948
  open("am_delta.xhuff","wb").write(raw[8:])               # strip 8-byte MAM header
  # am_delta.expected = decompress(am_delta.xhuff, size), confirmed byte-identical
  # by Fox-IT dissect.util (see docs/validation.md)
  PY
  ```
- **MD5**: `am_delta.xhuff` `f3548390c17ed5af3845ea830ea66d48`;
  `am_delta.expected` `193b1fc2f87f4fac2afeea27aaaeb085` (SHA-256 of the
  decompressed output:
  `36616a7fe2ead05d31877253563ac2199ea55465ed8c133bca67c448ad2811cc`).

#### audiodg.xhuff / audiodg.expected

- **What**: a second, larger Xpress-Huffman vector (9038 bytes compressed →
  35954 bytes), same provenance and generator as above but derived from
  `AUDIODG.EXE-AB22E9A6.pf` (same Stolen Szechuan Sauce Desktop image). Used by
  `decompresses_larger_real_vector`.

[MS-XCA]: https://learn.microsoft.com/en-us/openspecs/windows_protocols/ms-xca/
