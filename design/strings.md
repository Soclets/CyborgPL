# Text — the units, walking, and the toolbox

Decided 2026-09-20, one question at a time. A `str` is an owned value
(`str.utf8` / `utf16` / `utf32` / `ascii`, `types.md`); this file is what
you can *do* with one.

## A length bound names its unit (decided 2026-09-22)

```
let 'name': str.utf8[1, 40 graphemes] = std::input.line[] otherwise "";
fn greet [lent 'who': str.utf8[1, 10 graphemes]] { … }
let 'field': str.utf8[0, 255 bytes] = …;
let 'code': str.utf8[4, 4 points] = …;
```

The thing, then its bounds, as `int32[0, 100]` and `list[T][lo, hi]` —
and, because a text has three lengths and no single one, the bound
**says which it counts**: `bytes`, `points`, or `graphemes`, after the
high bound. `str.utf8[0, 20]` with no unit is a compile error. Every
change to a bounded text (assignment, `++`) is a check site against the
bound; a conversion into one (`to.str.utf8[0, 3 bytes][…]`) fails
`out_of_range` like any conversion into a range type. Precog carries
the length interval per unit already. Proposed: bytes always, graphemes
always, or say which; owner chose "say which".

## No single length

`"é🍎"` is 5 bytes, 2 code points, 2 graphemes; a family emoji is 18
bytes, 5 points, 1 grapheme. The language has the three units already
(`char.byte` / `.point` / `.grapheme`), so it has three counts and no
`str.length` — the thing that makes other languages cut emoji in half:

```
str.bytes[lend 's']        # what the OS and C see; O(1)
str.points[lend 's']       # Unicode scalar values
str.graphemes[lend 's']    # what a person counts as one character; O(n)
```

`str.length` is a compile error naming the three.

## Walking says the unit too

```
for 'g' in str.each.grapheme[lend 's'] { … }     # 'g': char.grapheme
for 'p' in str.each.point[lend 's'] { … }        # 'p': char.point
for 'b' in str.each.byte[lend 's'] { … }         # 'b': uint8
```

A bare `for 'c' in lend 's'` is an error: a text is not a list of anything
until you say what. `each.<unit>` yields the units without building a
list (Precog lowers it to a loop over the bytes; the oracle builds the
list).

## No `[pos]` on a text

`'s'[pos 3]` is a compile error — the 3rd byte or the 3rd grapheme? The
positional forms live in std with the unit spelled and a failure for out
of range: `std::str.at.grapheme[lend 's', 3] otherwise …`,
`std::str.slice.grapheme[lend 's', 1, 5] otherwise …`. Most "the 3rd
character" code is a `split` or a `find` in disguise.

## The toolbox is std's

The language owns only what the compiler must *know* — the units and how
to walk them. Everything else is ordinary library code, and case tables
change with every Unicode release, so they belong in something updatable:

```
std::str.split[lend 's', " "]          -> list[str.utf8]
std::str.join[lend 'parts', "_"]
std::str.contains / starts_with / ends_with[lend 's', "x"]   -> bool
std::str.trim / upper / lower[lend 's']
std::str.replace[lend 's', "a", "b"]
std::str.find.grapheme[lend 's', "x"]  -> int32 or fail[missing]   (1-based grapheme position)
std::str.at.<unit>[lend 's', 3]        -> that unit, or fail[out_of_range]
std::str.slice.grapheme[lend 's', 1, 5] -> str, or fail[out_of_range]
```

The oracle emulates these so examples run (`examples/text.cpls`); std's
real versions will be CyborgPL.

## Comparison

- `==` / `!==` on text compare **bytes exactly** — no normalisation.
  `"é"` as one code point is not `"é"` as `e` + combining accent;
  normalising is a std function. A `char` compares by content, whatever
  unit it was declared in.
- `<` / `>` / `<==` / `>==` on text are a **compile error**: byte order is
  never what anyone means, and a collation rule (locale-aware ordering)
  is std's, undesigned.

## Open

- A collation / locale-aware compare in std.
- Normalisation (NFC/NFD) in std.
- Building text piece by piece efficiently (a builder) — `++` allocates
  each time; Precog may fold chains, but a `list[str]` + `std::str.join`
  is the honest shape today.
