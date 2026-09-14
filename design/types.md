# Types

Owner's initial type family, 2026-09-14. Spelled as they appear in a chain
(`var.bin16`, `var.str`, …).

| Chain name | Meaning | Widths |
|---|---|---|
| `bin` | IEEE 754 binary floating point | 16, 32, 64, 128 (proposed by Claude; owner to confirm) |
| `deci` | IEEE 754 decimal floating point | 32, 64, 128 (IEEE 754-2008 defines only these three) |
| `int` | signed integer, two's complement | 8, 16, 32, 64, 128 (proposed) |
| `uint` | unsigned integer | 8, 16, 32, 64, 128 (proposed) |
| `str` | text | — |

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
- **Not yet listed:** a boolean; a character type (or is a one-character
  `str` enough?); a byte alias for `uint8`; arbitrary-precision numbers
  (legacy had `bignum`/`bigint` via GMP — do they return?).
