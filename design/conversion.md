# Conversion — `to.<type>[…]`

Decided 2026-09-19. Nothing converts by itself; a conversion is spelled as
`to`, the target type, and the value in brackets.

```
to.stored.str.utf8['avg']          # number → text, in its stored form; never fails
to.bin64['n']                      # int → bin, widening; never fails
to.int8['big']                     # narrowing: a check site (does it fit?), like a typed assignment
to.int32["42"] otherwise 0         # text → number: can fail with `not_a_number` or `out_of_range`
to.int32["7"] otherwise { not_a_number { std::exit[2]; } out_of_range { std::exit[3]; } }
```

- **`to`** is a keyword and the head of the chain, like `list` in
  `list.length[…]`; the chain narrows to the target type (`to.str.utf8`,
  never a bare `to.str`); the bracket holds exactly one value. Owner's
  spelling, over `int32[…]` (types-as-calls) and `std::to.int32[…]` (a
  library job): the compiler owns conversion — Precog has to know exactly
  what int→bin does to prove ranges — and a bare head reads as what it is.
- **Text that is a real number too big for the type fails `out_of_range`**
  (owner, 2026-09-24: c): `to.int32["99999999999"] otherwise …`, however
  many digits, and `to.uint8["-1"]` likewise. Text comes from outside, so a
  number that does not fit is bad input to handle, never a crash and never
  a check site, which the final build could not prove about outside text.
  Proposed alternatives: a check site like any narrowing; `not_a_number`
  past some size (the oracle's accidental 2^127 edge).
- **Which text is a bin** (owner, 2026-09-24: b): an optional sign,
  digits, an optional `.` and digits, and an optional exponent `e`/`E`
  with its own sign: `"3.14"`, `"-2"`, `"2.5e3"`. Anything else fails
  `not_a_number`, `"inf"` and `"nan"` included, so neither value ever
  enters a program from text. A data file's `nan` ("missing") lands in the
  writer's `otherwise`, which says what it means there. Proposed
  alternatives: the source grammar only, without exponents; whatever
  Rust's parser takes (the oracle's accident, with `inf`, `infinity`,
  `nan`); recognising `inf`/`nan` to fail `out_of_range`/`not_a_number`.
- **A conversion that can fail fails the normal way**: text → number is
  `-> … or fail[not_a_number, out_of_range]` and takes `otherwise` like any call
  (`failures.md`). A narrowing conversion is not a failure but a **check
  site**: `to.int8['big']` carries a runtime check unless Precog proves the
  fit, and `check to.int8[…]` asks for the proof.
- **Between `list` and `array`**: `to.array[give 'l']`, `to.list[copy
  'a']`, and the element type is not written (owner, 2026-09-23;
  `lists.md`).
- **Text into ASCII fails `not_ascii`** unless every character is ASCII
  (owner, 2026-09-30: "`otherwise` exists"): `to.str.ascii['t'] otherwise
  …`. From ASCII text, a number or a byte it cannot fail and takes no
  `otherwise`; into `utf16` or `utf32` never fails. The text encodings
  are the four of 2026-09-14 (`utf8`, `utf16`, `utf32`, `ascii`), each a
  conversion target. `examples/to_ascii.cpls`.
- **Bytes into text** (owner, 2026-09-30: "yea"): `to.str.utf8['bytes']`
  reads a `list[uint8]` or `array[uint8]` as UTF-8 and fails `not_utf8`
  when it is not: `[104, 195, 169, 108, 108, 111]` is `"héllo"` (`é` is
  the two bytes 195, 169). Only `utf8` reads bytes so far. C's text is
  walked by hand into such a list, then converted once
  (`examples/args.cpls`); a one-call shortcut is std's to write.
  `examples/bytes_to_text.cpls`. **Any other list or array is not text**
  (owner, 2026-09-30: b): no `"[1, 2]"`; join the items with `++`, or
  print the list.
- **`++` takes text on both sides.** This closes the language's last
  implicit conversion (two cold readers flagged it): `"n=" ++ 'n'` is an
  error pointing at `to.str.utf8`. `print`/`print.useful` still render any value
  handed to them directly — rendering for output is what they are for, not
  a conversion.
- A number becomes text in its **stored form**, always (2026-09-14):
  `to.stored.str.utf8[…]` is what `print.stored` shows. **The form is
  always written** (owner, 2026-10-02: "yes", matching `print.stored`):
  a bare `to.str.utf8[…]` of a number, `bool` or enum case is refused,
  naming both forms. Text, characters and bytes into text stay bare: they
  have one form. **`to.useful.str.utf8[…]` is
  what `print.useful` shows** (owner, 2026-09-30): `0.1` for the bin whose
  exact value is 55 digits, `1.5` for the deci `1.50`, and an enum case's
  bare word, `big`, where the exact form is `Size.big`. A `bool` is
  `true`/`false` in both. **`.useful` follows the verb** (owner: b, as in
  `print.useful`): `to.useful.str.utf8`, `to.useful.deci64` (moved from
  `to.deci64.useful`). A bin into a `deci` writes its form too,
  `to.stored.deci64` or `to.useful.deci64` (owner, 2026-10-02; `types.md`). `examples/to_text_forms.cpls`. **A struct, a map,
  or an enum case carrying data is not text** (owner, 2026-09-30:
  "refuse"): build the text from its parts, or print it directly. **An
  enum with any data-carrying case is not text at all**, even a value that
  is a plain case (owner: a): the rule comes from the type, as every
  conversion's does. Printing still shows any value.
- **bin → int fails unless the bin is whole** (decided 2026-09-21):
  `to.int32[2.0] otherwise …` is `2`; `to.int32[2.7]` fails `not_whole`.
  **A whole bin or deci too big for the integer fails `out_of_range`**
  (owner, 2026-09-30: a), as text does: `to.int32[10000000000.0]
  otherwise -1` is `-1`, and one `otherwise` handles both reasons.
  Nothing is thrown away silently — rounding is said first, and the
  reader sees where: `to.int32[std::round['x']]`. The rounding toolbox
  (`std::round`, `floor`, `ceil`, `trunc`: a bin in, a whole bin out) is
  std's, like `div.trunc`; names provisional. Owner chose this over C's
  quiet truncation and over floor (which would match `div`).
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
