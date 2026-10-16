# Types

> Revised 2026-10-07: code here is in the new syntax; an old spelling in the prose is history. The old → new table is `revision-2026-10-07.md`.

Owner's initial type family, 2026-09-14. Spelled as they appear in a chain
(`'x': bin16`, `'s': str.utf8`, …; the type follows the name since 2026-09-19). A type parameter goes in brackets — `list[int32]`, `array[str.utf8]` — while `.` picks a fixed variant of a type (decided 2026-09-19; see `lists.md`).

| Chain name | Meaning | Widths |
|---|---|---|
| `bin` | IEEE 754 binary floating point | 16, 32, 64 (owner, 2026-09-23: **no `bin128`** — see below) |
| `deci` | IEEE 754 decimal floating point | 32, 64, 128 (IEEE 754-2008 defines only these three) |
| `int` | signed integer, two's complement | 8, 16, 32, 64, 128 (confirmed 2026-09-23) |
| `uint` | unsigned integer | 8, 16, 32, 64, 128 (confirmed 2026-09-23) |
| `str` | text, narrowed by encoding: `'s': str.utf8` (decided 2026-09-14) | `utf8`, `utf16`, `utf32`, `ascii`; `:=` picks the best encoding from use |
| `bool` | boolean; what comparisons and `and`/`or`/`not` produce; literals `true` / `false` (decided 2026-09-14): `let 'correct?': bool = true;` | — |
| `char` | one character, narrowed by *unit*: `char.point`, `char.grapheme`, `char.byte` (decided 2026-09-14) | see below |

`bin16` is a half-precision float, not a 16-bit integer; `int16` is that.

**`brain16`, the brain float** (owner, 2026-10-02; in the oracle the same day), also
spelled `bfloat16` and `bf16`: Google Brain's 16-bit float for neural
networks — `bin32`'s 8 exponent bits, so its range, with 7 bits of
precision. Not IEEE 754, so not a `bin`. LLVM's type is `bfloat`. Its
arithmetic rounds after every step, as `bin16`'s does (owner,
2026-10-02: a). It is its own family, as `deci` is (owner: a): it mixes
with nothing without `to.` — `to.bin32['w']` exact, `to.brain16['x']`
rounding, `to.bin16['w']` a check site. A literal rounds to the nearest `brain16`
(owner, 2026-10-02: a). The GPU is open.

**`exactly`** (owner, 2026-10-14): `exactly 19.99`, the only spelling since 2026-10-16 (the prefix `*`, `*19.99`,
is an old spelling: a symbol spells only an operator, owner: "Yeah, drop it too"). It was the word for `*19.99`, and both took a whole
number too, `exactly 1000` being a `deci` holding 1000 (until then `*1000` was refused as exact already).

**`*0.1` means "exactly this"** (owner, 2026-10-02; in the oracle the same day): a
decimal literal marked `*` is a `deci`, `let 'price' := *19.99;`; a plain
one is a `bin`, meaning `bin64`'s answer: the prover narrows it only where
it proves the whole program gives the same results (owner, 2026-10-02),
so `brain16` is never chosen, only written.

**Removed 2026-10-07** (owner: "MTM is gone."): Rust's names are refused, naming
CyborgPL's (`i32` → "write `int32`"). What follows is their history. **Rust's names
are aliases** (owner, 2026-10-02; in the oracle the same day): `i8`–`i128`
for `int8`–`int128`, `u8`–`u128` for `uint8`–`uint128`, `f16` `f32` `f64`
for `bin16` `bin32` `bin64`. Nothing else: not `bool` (the same word),
not `char` (a unit must be said), no `deci` (Rust has none), not
`String`, `&str`, `usize`, `isize` or `Vec<T>` (`DECISIONS.md`). Only these and CyborgPL's
own names (owner, same day: "only include the Rust ones and ours"): C's
exact names, recorded first, were withdrawn, and no long spellings
(`binary_float16`) are added.

**`bin128` is back** (owner, 2026-10-10: "It wants both", with
`not_std.bin80`, for C's `long double`; DECISIONS has the rest). What
follows is the 2026-09-23 drop it reverses, kept as history.

**No `bin128`** (owner, 2026-09-23: "b", over keeping it). The reason was
design claude's, given when proposing it: quad precision has essentially
no hardware anywhere — it is software arithmetic, and slow — and the thing
people reach for it for is "64 bits is not enough precision", which
`std::uenr` answers properly by being exact rather than merely wider.
`deci128` stays, because IEEE 754 defines exactly three decimal widths
and dropping the largest would leave a gap in a standard rather than
remove a redundancy. `int128` and `uint128` stay: no CPU has 128-bit
registers either, but they are built from two 64s cheaply, they are
what hashes and big counters want, and `std::uenr` is built on them.

Owner's condition, recorded as a requirement rather than a wish:
**`std::uenr` had better be optimised.** It is now the only answer for
precision beyond 64 bits, so its performance is not a nice-to-have —
see `implementation-languages.md`, where the runtime, the runtime, is `no_std` Rust
and `deci`/`uenr` are ours to write rather than vendored.

## Integers of any width: `!std.int24` (owner, 2026-10-03; the `.` since 2026-10-06)

```
let 'sample': not_std.int24 = -8_000_000;     # Rust-style: !std.i24
let 'flags': not_std.uint4 = 9;               # !std.u4
```

Any width LLVM takes (from 1 bit to its own limit, about 8 million),
always marked `!std.`, "not standard": `!std.intN` / `!std.uintN`, or
Rust-style `!std.iN` / `!std.uN` (Rust itself stops at 128; LLVM and Zig
have them). The standard widths stay plain, and the mark on one is
refused (`!std.int32` → `int32`). Each holds exactly its width's range
and is checked where it lands like any integer; `.wrap` wraps at its
width. Proposed: whole bytes only, or a short list, and a cap of 4,096
bits; the owner chose any width ("there's already a disclaimer in
front"). LLVM's widest is 8,388,608 bits (2^23); past it is refused. In
between, the maths is limited by the widest operand's width, at least 64
bits (`operators.md`). `examples/any_width.cy`,
`examples/wide_integers.cy`.

## A declaration always has a value (owner, 2026-09-23)

`let 'n': int32;` is refused — write the `0`. There is no implicit
zero, for any type: nothing is ever silently `0`, `false`, `""` or an
empty list, so no value ever appears from nowhere and `:=` always has
something to infer from. Go, C# and Java fill zeroes in for fields;
this does not.

## `deci` (owner, 2026-09-29)

IEEE 754 decimal: `deci32` holds 7 digits, `deci64` 16, `deci128` 34, and
`0.1 plus 0.2` is exactly `0.3`.

- **A literal with more digits than the type holds is a compile error**:
  `let 'q': deci32 = 3.14159265;` is refused ("9 digits; `deci32` holds 7").
- **Each operation rounds to its width, half to even**, IEEE's default,
  as `bin` does. With 16 digits, money rarely reaches it; a receipt's
  rounding is one explicit call at the end (std's, undesigned).
- **From `bin`** (and `brain16`), **the form is always written** (owner,
  2026-10-02: "Ah, opt 1", as for text): `to.stored.deci64['w']` gives the
  exact value rounded to the width (a `bin64` 0.1 is `0.1000000000000000`
  in a `deci64`, and shows its binary tail in a `deci128`);
  `to.useful.deci64['w']` gives the shortest decimal that reads back as
  the same `bin` (`0.1`). A bare `to.deci64['w']` of a bin is refused,
  naming both. From an integer, a `deci` or text it stays bare: one
  value.
- **From an integer**: `to.deci64['c'] otherwise …` fails
  `too_many_digits` when the integer's type could have more digits than
  the `deci` holds (an `int64` into a `deci64`); an `int32` into a
  `deci64` always fits and takes no `otherwise`.
- **From text**: `bin`'s grammar; more digits than the width fails
  `too_many_digits`, beside `not_a_number` and `out_of_range`. **By the
  value**: `to.deci32["1234567.0"]` is 1234567 and fits, a trailing zero
  dropped where it must be; **`to.deci32.written[…]`** counts every digit as
  written, trailing zeros included (owner, 2026-10-09).
- **Trailing zeros are kept**, as IEEE does: `1.50` stays `1.50`, `2.50 x
  2` is `5.00`, `1.50 == 1.5` is true. `print` shows `1.50`, `print.useful`
  `1.5`.
- **The bit layout is each machine's own**: DPD on IBM POWER and z (their
  decimal hardware), BID elsewhere (as C's `_Decimal64` there), so a `deci`
  handed to C always matches.
- **To an integer** fails `not_whole` unless whole, and `out_of_range`
  when whole but too big (owner, 2026-09-30: a); **to a `bin`** rounds, as
  `bin` math does.
- `deci` is its own family: it never mixes with `bin` or an integer
  without `to.…` (2026-09-14).
- **`mod.floor` takes the divisor's sign**, as integer `mod.floor`: `-7.50 mod.floor 2` is
  `0.50`. Exact, then rounded like the rest.
- **`pow` takes a whole exponent** and rounds like the other operations:
  `1.05 pow 10` is `1.628894626777441` in a `deci64`, exact in a `deci128`; a
  negative exponent is one over the power. A fractional one (`2.0 pow 0.5`)
  is refused: roots are std's math. The exponent is an integer or a whole
  literal; a `deci` variable is refused even when it holds `3.00`, since the
  compiler cannot know it is whole (owner, 2026-09-29: a): `'r' pow
  to.int32['k'] otherwise …`.
- **A wider `deci` into a narrower one fails `too_many_digits`** when its
  digits do not fit (`to.deci32['x'] otherwise …`); `2.50` converts exactly.
- **A `deci` may be a range type**, with `bin`'s rules: `deci64[min 0.0, max 1.0]`,
  the bounds written with a point; a value outside is a check site, and
  `to.deci64[min 0.0, max 1.0][…]` fails `out_of_range`.

## Notes (Claude, 2026-09-14)

- **Four numeric families, no implicit conversion between them.** `int32`
  and `bin32` are different families; mixing them in one expression is a
  compile error and conversion is explicit (spelling open). A bare numeric
  **literal** has no family of its own: it takes the family of its context,
  so `let 'x': bin16 = 1000;` and `'b' times 2` with `'b'` a bin are fine. A
  *variable* never converts: `let 'b': bin64 = 'a';` with `'a'` an `int32`
  is an error.
- **Width in the typed form is mandatory** (`int32`, never `int`) —
  decided 2026-09-14; see "typed form is always fully explicit" below.
- **Hardware reality**, for the LLVM backend: `bin16/32/64` are native on
  current targets — which is the whole of `bin` now that `bin128` is gone.
  `deci*` is hardware on IBM POWER and z/Architecture
  (the owner's other language, Xag, already lowers to those DFP instructions)
  and software everywhere else. LLVM IR has no decimal type, so the hardware
  path must go through target intrinsics or inline asm rather than a plain IR
  op.
- **`str` layout** (owner, 2026-09-22): a **16-byte handle — a pointer
  and a length** — plus the bytes it points at. No capacity field: a
  text is never grown in place, since `++` makes a new one, so a third
  number would be dead weight in every list, struct and array of texts.
  **Where the prover proves a text is built by repeated `++`** — the
  builder pattern, a loop appending to the same name — *that* text
  gets a hidden capacity and grows in place, one allocation instead of
  N. The field exists only in the values that earn it; the type stays
  16 bytes. (Proposed: 16 bytes always; 24 bytes always; owner: both,
  the prover deciding — the same trick as `list` vs `array` and a map's
  order list.) Known-at-compile-time texts need no heap: the prover bakes
  them into the binary as constant data. **Which memory a text points
  at is the prover's to know, not the value's to carry** (owner,
  2026-09-22): it emits a free only where the value can actually be on
  the heap — a name that only ever holds constants is never freed, and
  no runtime test is paid. Where a name may hold either (a constant in
  one branch, a built text in the other) the handle carries a sentinel
  — the high bit of the length — and the free tests it. (Proposed: the
  sentinel always, one test per drop; copying every constant to the
  heap on first use, which allocates in exactly the programs that wrote
  a literal to avoid allocating.)
- **`str` encoding is part of the chain** (`str.utf8`), same shape as a
  numeric width. Encodings: `utf8`, `utf16`, `utf32`, `ascii`; `:=` picks the best from use;
  in the typed form the encoding is mandatory (decided 2026-09-14).
- **`char` units.** The chain narrows `char` by what "one character" means,
  because Unicode has three honest answers:
  - `char.byte` — one byte. 1 byte, flat. For ASCII and raw data.
  - `char.point` — one Unicode code point. 4 bytes, flat, plain copy, no
    heap, stable across Unicode versions. The fast internal unit.
  - `char.grapheme` — one grapheme cluster: what a human sees as one
    character (`กี่`, a consonant with its vowel and tone mark; `👨‍👩‍👧`;
    `e`+combining accent). A grapheme is not a syllable: `เกี่ยว` is four,
    เ | กี่ | ย | ว (UAX #29; corrected 2026-09-24, it said two).
    Variable size — small inline buffer with heap fallback, so it is an
    *owned* value like `str`; needs Unicode segmentation tables in the
    binary; boundaries can shift with Unicode versions, so the version is
    pinned per build and updated deliberately (`reproducible-builds.md`). The correct
    user-facing unit.
  A character literal is written in **backticks**: `` let 'c' := `1`; ``
  (decided 2026-09-14). Three quote styles, three token classes: `"…"` text,
  `'…'` name, `` `…` `` one character. The literal must be exactly one unit
  of the chosen or inferred kind. With `:=`, the prover picks the unit from the
  content and its use (`` `a` `` may be a byte; `` `🍎` `` needs `point`;
  `` `กี่` `` can only be `grapheme`).
- **The typed form is always fully explicit** (decided 2026-09-14): no bare
  `int`, `str`, or `char`. `int32`, `str.utf8`,
  `char.point`. `:=` means the prover chooses; `=` means you said
  everything.
- **`uenr` — Unbounded Exact Numerical Representation** (owner, 2026-09-14). A number
  type with no fixed width that never rounds: it grows to hold whatever
  value it is given. Provided by the **standard kit**, not built into the
  language — so it is CyborgPL source the prover can see through, and it takes
  the role legacy's `bignum`/`bigint` had. Reached through the kit
  boundary: `let 'n': std::uenr = 1000;`. **Exact is exact**: fractions are
  held as true rationals, so `(1/3) times 3` is exactly `1` (decided
  2026-09-14).
