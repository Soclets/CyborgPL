# Conversion — `to.<type>[…]`

Decided 2026-09-19. Nothing converts by itself; a conversion is spelled as
`to`, the target type, and the value in brackets.

```
to.str.utf8['avg']                 # number → text, in its stored form; never fails
to.bin64['n']                      # int → bin, widening; never fails
to.int8['big']                     # narrowing: a check site (does it fit?), like a typed assignment
to.int32["42"] otherwise 0         # text → number: can fail with `not_a_number`
to.int32["7"] otherwise { not_a_number { std::exit[2]; } }
```

- **`to`** is a keyword and the head of the chain, like `list` in
  `list.length[…]`; the chain narrows to the target type (`to.str.utf8`,
  never a bare `to.str`); the bracket holds exactly one value. Owner's
  spelling, over `int32[…]` (types-as-calls) and `std::to.int32[…]` (a
  library job): the compiler owns conversion — Precog has to know exactly
  what int→bin does to prove ranges — and a bare head reads as what it is.
- **A conversion that can fail fails the normal way**: text → number is
  `-> … or fail[not_a_number]` and takes `otherwise` like any call
  (`failures.md`). A narrowing conversion is not a failure but a **check
  site**: `to.int8['big']` carries a runtime check unless Precog proves the
  fit, and `check to.int8[…]` asks for the proof.
- **Between `list` and `array`**: `to.array[give 'l']`, `to.list[copy
  'a']`, and the element type is not written (owner, 2026-09-23;
  `lists.md`).
- **`++` takes text on both sides.** This closes the language's last
  implicit conversion (two cold readers flagged it): `"n=" ++ 'n'` is an
  error pointing at `to.str.utf8`. `print`/`printu` still render any value
  handed to them directly — rendering for output is what they are for, not
  a conversion.
- A number becomes text in its **stored form**, always (2026-09-14). A
  useful-form conversion (`0.1` rather than the 55 digits) has no spelling
  yet — `printu` is the only thing that produces it.
- **bin → int fails unless the bin is whole** (decided 2026-09-21):
  `to.int32[2.0] otherwise …` is `2`; `to.int32[2.7]` fails `not_whole`.
  Nothing is thrown away silently — rounding is said first, and the
  reader sees where: `to.int32[std::round['x']]`. The rounding toolbox
  (`std::round`, `floor`, `ceil`, `trunc`: a bin in, a whole bin out) is
  std's, like `div.trunc`; names provisional. Owner chose this over C's
  quiet truncation and over floor (which would match `/`).
  `examples/whole.cpls`.
- **`char`** (decided 2026-09-21, three questions):
  - **Number ↔ char is not a conversion.** `to.int32[`7`]` is refused: it
    could mean 55 (the code) or 7 (the digit), and C's answer is the trap.
    Two named operations instead: `char.code[`A`]` → `65` (a `uint32`
    for a point, `uint8` for a byte; a grapheme has no single code and is
    refused), and `char.point.from_code[65] otherwise …` /
    `char.byte.from_code[…] otherwise …` → the char, failing `not_a_char`
    for a code that is not one. A digit's value is a text parse, spelled
    as one: `to.int32[to.str.utf8[`7`]] otherwise …`. Owner chose this
    over "`to.int32` gives the code" and "`to.int32` gives the digit".
  - **Text → char is a conversion that can fail**: `to.char.point["A"]
    otherwise …` gives `` `A` ``; `"AB"` or `""` fails `not_one_char`.
    The unit in the target says what "one" means
    (`to.char.grapheme["เกี่ยว"]` fails: several graphemes). Char → text
    is `to.str.utf8[…]`, never fails. Owner chose this over "use
    `std::str.at.<unit>`".
  - **Between units, down can fail, up never does**: `to.char.point[g]`
    from a grapheme fails `not_one_point` unless it is a single point;
    `to.char.byte[p]` fails `not_one_byte` unless it is ASCII;
    `to.char.grapheme[p]` and `to.char.point[b]` always work and take no
    `otherwise`. Same shape as `not_whole`.
  `examples/chars.cpls`.
- Not decided, refused by the oracle: anything → `bool` (no truthiness —
  compare instead); to or from lists, structs.
