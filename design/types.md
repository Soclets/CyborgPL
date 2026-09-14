# Types

Owner's initial type family, 2026-09-14. Spelled as they appear in a chain
(`var.bin16`, `var.str`, …).

| Chain name | Meaning | Widths |
|---|---|---|
| `bin` | IEEE 754 binary floating point | 16, 32, 64, 128 (proposed by Claude; owner to confirm) |
| `deci` | IEEE 754 decimal floating point | 32, 64, 128 (IEEE 754-2008 defines only these three) |
| `int` | signed integer, two's complement | 8, 16, 32, 64, 128 (proposed) |
| `uint` | unsigned integer | 8, 16, 32, 64, 128 (proposed) |
| `str` | text, narrowed by encoding: `var.str.utf8` (decided 2026-09-14) | `utf8`, `utf16`, `utf32`, `ascii`; `:=` picks the best encoding from use |
| `bool` | boolean; what comparisons and `and`/`or`/`not` produce | — |

`bin16` is a half-precision float, not a 16-bit integer; `int16` is that.

## Notes (Claude, 2026-09-14)

- **Four numeric families, no implicit conversion between them.** `int32`
  and `bin32` are different families; mixing them in one expression is a
  compile error and conversion is explicit (spelling open).
- **Width in the typed form.** With `:=` the compiler picks the width, so a
  bare `int` is never needed there. Recommendation: in the explicit form the
  width is mandatory (`var.int32`, never `var.int`) — the explicit form only
  appears at boundaries, where being precise is the point.
- **Hardware reality**, for the LLVM backend: `bin16/32/64` are native on
  current targets; `bin128` is software (legacy CyborgPL had a from-scratch
  quad implementation). `deci*` is hardware on IBM POWER and z/Architecture
  (the owner's other language, Xag, already lowers to those DFP instructions)
  and software everywhere else. LLVM IR has no decimal type, so the hardware
  path must go through target intrinsics or inline asm rather than a plain IR
  op.
- **`str` layout:** a fixed 24-byte handle (pointer, length, capacity) plus
  a heap buffer sized by the encoding. Known-at-compile-time strings need no
  heap — Precog bakes them into the binary as constant data.
- **`str` encoding is part of the chain** (`var.str.utf8`), same shape as a
  numeric width. Encodings: `utf8`, `utf16`, `utf32`, `ascii`; `:=` picks the best from use
  (decided 2026-09-14). Open: whether the encoding is mandatory in the typed
  form like a width (recommended: yes).
- **Not yet listed:** a character type (or is a one-character
  `str` enough?); a byte alias for `uint8`; arbitrary-precision numbers
  (legacy had `bignum`/`bigint` via GMP — do they return?).
