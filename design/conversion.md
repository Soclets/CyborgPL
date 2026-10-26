# Conversion — `to.<type>[…]`

> Revised 2026-10-07: code here is in the new syntax; an old spelling in the prose is history. The old → new table is `revision-2026-10-07.md`.

Decided 2026-09-19. Nothing converts by itself; a conversion is spelled as
`to`, the target type, and the value in brackets.

```
to.stored.str.utf8($avg)          # number → text, in its stored form; never fails
to.f64($n)                      # int → bin, widening; never fails
to.i8($big)                     # narrowing: a check site (does it fit?), like a typed assignment
to.i32("42") otherwise 0         # text → number: can fail with `not_a_number` or `out_of_range`
to.i32("7") otherwise { not_a_number { std::exit(code: 2); } out_of_range { std::exit(code: 3); } }
```

- **`to`** is a keyword and the head of the chain, like `list` in
  `list.length[…]`; the chain narrows to the target type (`to.str.utf8`,
  never a bare `to.str`); the bracket holds exactly one value. Owner's
  spelling, over `i32[…]` (types-as-calls) and `std::to.i32[…]` (a
  library job): the compiler owns conversion — the prover has to know exactly
  what int→bin does to prove ranges — and a bare head reads as what it is.
- **Text that is a real number too big for the type fails `out_of_range`**
  (owner, 2026-09-24: c): `to.i32["99999999999"] otherwise …`, however
  many digits, and `to.u8["-1"]` likewise. Text comes from outside, so a
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
  `--> … or fail[not_a_number, out_of_range]` and takes `otherwise` like any call
  (`failures.md`). A narrowing conversion is not a failure but a **check
  site**: `to.i8['big']` carries a runtime check unless the prover proves the
  fit, and `check to.i8[…]` asks for the proof.
- **Between `list` and `array`**: `to.array[give 'l']`, `to.list[copy
  'a']`, and the element type is not written (owner, 2026-09-23;
  `lists.md`).
- **Text into ASCII fails `not_ascii`** unless every character is ASCII
  (owner, 2026-09-30: "`otherwise` exists"): `to.str.ascii['t'] otherwise
  …`. From ASCII text, a number or a byte it cannot fail and takes no
  `otherwise`; into `utf16` or `utf32` never fails. The text encodings
  are the four of 2026-09-14 (`utf8`, `utf16`, `utf32`, `ascii`), each a
  conversion target. `examples/to_ascii.cy`.
- **Bytes into text** (owner, 2026-09-30: "yea"): `to.str.utf8['bytes']`
  reads a `list[u8]` or `array[u8]` as UTF-8 and fails `not_utf8`
  when it is not: `[104, 195, 169, 108, 108, 111]` is `"héllo"` (`é` is
  the two bytes 195, 169). Only `utf8` reads bytes so far. C's text is
  walked by hand into such a list, then converted once
  (`examples/args.cy`); a one-call shortcut is std's to write.
  `examples/bytes_to_text.cy`. **Any other list or array is not text**
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
  `print.useful`): `to.useful.str.utf8`, `to.useful.d64` (moved from
  `to.d64.useful`). A bin into a `deci` writes its form too,
  `to.stored.d64` or `to.useful.d64` (owner, 2026-10-02; `types.md`). `examples/to_text_forms.cy`. **A struct, a map,
  or an enum case carrying data is not text** (owner, 2026-09-30:
  "refuse"): build the text from its parts, or print it directly. **An
  enum with any data-carrying case is not text at all**, even a value that
  is a plain case (owner: a): the rule comes from the type, as every
  conversion's does. Printing still shows any value.
- **bin → int fails unless the bin is whole** (decided 2026-09-21):
  `to.i32[2.0] otherwise …` is `2`; `to.i32[2.7]` fails `not_whole`.
  **A whole bin or deci too big for the integer fails `out_of_range`**
  (owner, 2026-09-30: a), as text does: `to.int32[10000000000.0]
  otherwise -1` is `-1`, and one `otherwise` handles both reasons.
  Nothing is thrown away silently — rounding is said first, and the
  reader sees where: `to.i32[std::round.near['x']]`. Rounding is std's, one family (owner,
  2026-10-17: "(c)"): `round.near` (a tie to even), `round.half_up` (a tie away from zero), `round.floor`,
  `round.ceil`, `round.trunc`, each `[value]` for a whole number or `[value, places: N]`, as `max_decimals`
  needs (types.md). A bare `round` is refused, since it names no way to round a tie (owner: "(a)"); the old
  `std::round`, `floor`, `ceil` and `trunc` are refused naming theirs. A bin is rounded from its exact digits
  (`round.near[2.675, places: 2]` is 2.67), and the answer is the kind that went in (design claude's readings). Owner chose this over C's
  quiet truncation and over floor (which would match `div`).
  `examples/whole.cy`.
- **`char`** (decided 2026-09-21, three questions):
  - **Number ↔ char is not a conversion.** `to.i32[`7`]` is refused: it
    could mean 55 (the code) or 7 (the digit), and C's answer is the trap.
    Two named operations instead: `char.code[`A`]` → `65` (a `u32`
    for a point, `u8` for a byte; a grapheme has no single code and is
    refused), and `char.point.from_code[65] otherwise …` /
    `char.byte.from_code[…] otherwise …` → the char, failing `not_a_char`
    for a code that is not one. A digit's value is a text parse, spelled
    as one: `to.i32[to.str.utf8[`7`]] otherwise …`. Owner chose this
    over "`to.i32` gives the code" and "`to.i32` gives the digit".
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
  `examples/chars.cy`.
- Not decided, refused by the oracle: anything → `bool` (no truthiness —
  compare instead); to or from lists, structs.

## A typed literal — `5: u8` (owner, 2026-10-17)

A literal may carry its type after `:`, as a name does in `'x': u8`. The `:` covers only the literal right
before it, so no brackets are needed inside maths (owner: "(a)", over brackets required):

```
let $r: i64 = to.i64(1) shl (5: u8 shr 1);   # the count gives no width, so the literal says it
let $a := 200: u8;
let $p := 5: u64[min: 1];
let $d := exactly 0.1: %d64;
```

- **Literals only.** A variable is converted with `to.u8['x']`; `'x': u8` in an expression is refused,
  naming the conversion.
- **It must fit, shown when building**: `300: u8` and `0: u64[min: 1]` are refused, so it never fails
  when it runs and needs no `otherwise`.
- Design claude's readings, not asked: a leading `-` belongs to the literal (`-128: i8`); a whole literal may
  take an int, bin or deci type; `1.5: i32` is refused (not whole); `0.1: d64` is refused, pointing at
  `exactly 0.1: d64`, since what a bin literal's digits mean as a deci is not decided; a typed `exactly`
  takes only a deci type; any type that is not a number is refused.
