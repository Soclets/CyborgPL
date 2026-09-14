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
| `char` | one character, narrowed by *unit*: `var.char.point`, `var.char.grapheme`, `var.char.byte` (decided 2026-09-14) | see below |

`bin16` is a half-precision float, not a 16-bit integer; `int16` is that.

## Notes (Claude, 2026-09-14)

- **Four numeric families, no implicit conversion between them.** `int32`
  and `bin32` are different families; mixing them in one expression is a
  compile error and conversion is explicit (spelling open).
- **Width in the typed form is mandatory** (`var.int32`, never `var.int`) —
  decided 2026-09-14; see "typed form is always fully explicit" below.
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
  numeric width. Encodings: `utf8`, `utf16`, `utf32`, `ascii`; `:=` picks the best from use;
  in the typed form the encoding is mandatory (decided 2026-09-14).
- **`char` units.** The chain narrows `char` by what "one character" means,
  because Unicode has three honest answers:
  - `char.byte` — one byte. 1 byte, flat. For ASCII and raw data.
  - `char.point` — one Unicode code point. 4 bytes, flat, plain copy, no
    heap, stable across Unicode versions. The fast internal unit.
  - `char.grapheme` — one grapheme cluster: what a human sees as one
    character (`เกี่ยว`'s syllables, `👨‍👩‍👧`, `e`+combining accent).
    Variable size — small inline buffer with heap fallback, so it is an
    *owned* value like `str`; needs Unicode segmentation tables in the
    binary; boundaries can shift with Unicode versions. The correct
    user-facing unit.
  A character literal is written in **backticks**: `` var 'c' := [`1`]; ``
  (decided 2026-09-14). Three quote styles, three token classes: `"…"` text,
  `'…'` name, `` `…` `` one character. The literal must be exactly one unit
  of the chosen or inferred kind. With `:=`, Precog picks the unit from the
  content and its use (`` `a` `` may be a byte; `` `🍎` `` needs `point`;
  `` `เกี่ยว` `` can only be `grapheme`).
- **The typed form is always fully explicit** (decided 2026-09-14): no bare
  `var.int`, `var.str`, or `var.char`. `var.int32`, `var.str.utf8`,
  `var.char.point`. `:=` means Precog chooses; `=` means you said
  everything.
- **`uer` — Unbounded Exact Representation** (owner, 2026-09-14). A number
  type with no fixed width that never rounds: it grows to hold whatever
  value it is given. Provided by the **standard Limb**, not built into the
  language — so it is CyborgPL source Precog can see through, and it takes
  the role legacy's `bignum`/`bigint` had. Reached through the Limb
  boundary: `var.std::uer 'n' = [1000];`. **Exact is exact**: fractions are
  held as true rationals, so `[1/3 x 3]` is exactly `1` (decided
  2026-09-14).
