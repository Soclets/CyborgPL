# Text — the units, walking, and the toolbox

> Revised 2026-10-07: code here is in the new syntax; an old spelling in the prose is history. The old → new table is `revision-2026-10-07.md`.

Decided 2026-09-20, one question at a time. A `str` is an owned value
(`str.utf8` / `utf16` / `utf32` / `ascii`, `types.md`); this file is what
you can *do* with one.

## What is inside `""` is exactly what is stored (restated 2026-09-22)

The quotes are not part of the text, and nothing inside them means
anything but itself — `"hello\n"` is seven bytes ending in a backslash
and an `n`, not six ending in a newline. The only two escapes are `\"`
and `\\`. A newline is written outside the quotes and joined:
`"hello" ++ \n`, which is the same six bytes C's `"hello\n"` makes.
(Restated after Claude wrote `"hello\n"` in an example and the owner
asked whether it works; it doesn't — the oracle stores the backslash.)

## `std::print.useful.auto_newline` — the line ended for you (owner, 2026-09-30)

```
std::print.useful.auto_newln["Hello, World!"];
```

`print.useful` with a newline after the values, the same as writing `\n`
last. The name says what it adds, where `println`'s `ln` hides it. It is
the first chain word with `_` in it. The owner wrote it "for fun", after
comparing the Hello World with Java's; `std::rs.println` (2026-09-19,
removed a day later) had done the same as a borrowed Rust print. Only this
form is decided: `print` and `print.format` have no `auto_newline`.
`examples/print_auto_newline.zg`.

## `std::print.weird` — C++'s `cout <<`, with arrows (owner, 2026-10-01)

```
std::print.weird.useful <- "tenth is " <- 'tenth' <- ", n is " <- 'n' <- newln;
std::print.weird.stored <- "stored: " <- 'tenth' <- newln;
```

A print in the shape of C++'s `std::cout << …` (owner: "`std::print.weird`
seems like a good name, maybe use `->` arrows"). Each value follows a
**`<--`**, pointing into the output, as C++'s `<<` points into `cout`
and as `<-` points into a variable in `inline.ASM` (owner: "`<-`", over
`-->`, which would point away from the screen — Zag's arrows point
where the value goes). It takes no `[ ]`, and at least one value. **`.weird.stored`**
shows the stored form, as `print.stored`; **`.weird.useful`** the
useful form, as `print.useful` (a bare `.weird` said "stored" until
2026-10-02, when every print came to say which) (owner: "what if we add a way to use
`.useful` with `.weird`?", then "yea"). The strongest modifier first:
`.weird` changes the whole call's shape, `.useful` only how values
look, so `print.useful.weird` is refused. The first borrowed-language
print since `std::c.printf` / `std::rs.println` were removed
(2026-09-20). `examples/print_weird.zg`; reject tests
print_weird_brackets, print_weird_order.

## `std::print.format` — the formatting print, with `{}` holes (2026-09-22; holes since 2026-10-01)

```
std::print.format["{}: {} at ${:.2}, 50% off\n", 'name', 'count', 'price'];
std::print.format["0x{:x} 0b{:08b} [{:>6}] [{:+}]\n", 255, 10, 42, 5];
std::print.format["{2} vs {1}, and {2} wins\n", "a", "b"];
std::print.format.stderr["error: {}\n", 'msg'];
```

The third print, beside `print.stored` (stored form) and `print.useful` (useful
form), and the only one that shapes its output — hex, padding, places —
so it keeps its own job (owner: "print is print, print useful is print
useful"). Decided 2026-10-01, replacing C's `%` codes (owner: "yea, {}
holes"):

- **A hole is `{}`**: the next value, in its **readable form**, as
  `print.useful` shows it (owner: "If you can change it inside, then go
  with readable"). The compiler knows every value's type, so no type
  letters.
- **Numbered holes** `{1}`, `{2}`… take a value by position, to use one
  twice (owner: "allow num"). They count **from 1**, as positions do —
  `{0}` is refused, like `'xs'[0]` (owner: "Position 0 doesn't exist").
  A format uses plain or numbered holes, **never both** (owner: "no
  mixing"). Every value fills a hole and every hole has a value, or it is
  a compile error.
- **A shape follows `:`**, Rust's set without `#`: fill and alignment
  `<` `^` `>` (`{:*^7}`), `+` for a sign, `0` padding, a width, `.places`
  (for text, the first characters), and a kind — `x` `X` `o` `b` for bases,
  `e` for scientific, and **`?` for the stored form** (`{:?}` of a `bin`
  `0.1` is `0.1000000000000000055…`). `#` is refused: type the prefix,
  `0x{:x}` (owner: "drop it"; `#` already means comments and `#strict`).
  `{:.2}` of a `deci` is exact, from its decimal digits, half to even.
- **Lists, structs and maps** fill a plain `{}` or `{:?}`, as `print.useful`
  and `print` show them; number shapes are refused for them (owner:
  "allow").
- **The format text has its own escapes, read in one pass** (owner:
  "Format print has its own rules"): `\n \t \r \\ \" \' \0`, `\xHH`,
  `\uHHHH`, `\UHHHHHHHH`, and **`\{` `\}`** for literal braces (owner:
  "`\`", over Rust's `{{ }}`). The language's own text rules do not run on
  it first, so `\\` is one backslash: `"C:\\temp"` prints `C:\temp`. An
  unknown escape is an error (owner: "Fine") — so `\%` is refused and
  `\\%` prints a backslash and a percent. **`%` is plain text.**
- A format that is not a literal cannot be checked and is a `no_check`
  site; it is read as stored text.

Before 2026-10-01 it was C's `printf` language (`%d %x %.2f`, `%%`, then
`\%`), named `printf` until 2026-09-29. The borrowed `std::c.printf` /
`std::rs.println` of 2026-09-19 stay removed. `examples/print_format.zg`;
reject tests `print_format_*`.

## A length bound names its unit (decided 2026-09-22)

```
let 'name': str.utf8[min 1, max 40 graphemes] = std::input.line[] otherwise "";
fn greet [lent 'who': str.utf8[min 1, max 10 graphemes]] { … }
let 'field': str.utf8[min 0, max 255 bytes] = …;
let 'code': str.utf8[min 4, max 4 points] = …;
```

The thing, then its bounds, as `int32[min 0, max 100]` and `list[T][lo, hi]` —
and, because a text has three lengths and no single one, the bound
**says which it counts**: `bytes`, `points`, or `graphemes`, after the
high bound. `str.utf8[0, 20]` with no unit is a compile error. Every
change to a bounded text (assignment, `++`) is a check site against the
bound; a conversion into one (`to.str.utf8[min 0, max 3 bytes][…]`) fails
`out_of_range` like any conversion into a range type. the prover carries
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
list (the prover lowers it to a loop over the bytes; the oracle builds the
list).

## No `[i]` on a text

`'s'[3]` is a compile error — the 3rd byte or the 3rd grapheme? The
positional forms live in std with the unit spelled and a failure for out
of range: `std::str.at.grapheme[lend 's', pos 3] otherwise …`,
`std::str.slice.grapheme[lend 's', from pos 1, to pos 5] otherwise …`. Most "the 3rd
character" code is a `split` or a `find` in disguise.

## The toolbox is std's

The language owns only what the compiler must *know* — the units and how
to walk them. Everything else is ordinary library code, and case tables
change with every Unicode release, so they belong in something updatable:

```
std::str.split[lend 's', at " "]          -> list[str.utf8]
std::str.join[lend 'parts', between "_"]
std::str.contains / starts_with / ends_with[lend 's', "x"]   -> bool
std::str.trim / upper / lower[lend 's']
std::str.replace[lend 's', from "a", to "b"]
std::str.find.grapheme[lend 's', part "x"]  -> int32 or fail[missing]   (1-based grapheme position)
std::str.at.<unit>[lend 's', pos 3]        -> that unit, or fail[out_of_range]
std::str.slice.grapheme[lend 's', from pos 1, to pos 5] -> str, or fail[out_of_range]
```

The oracle emulates these so examples run (`examples/text.zg`); std's
real versions will be Zag.

## Comparison

- `==` / `!==` on text compare **bytes exactly** — no normalisation.
  `"é"` as one code point is not `"é"` as `e` + combining accent;
  normalising is a std function. A `char` compares by content, whatever
  unit it was declared in.
- `<` / `>` / `</==` / `>/==` on text are a **compile error**: byte order is
  never what anyone means, and a collation rule (locale-aware ordering)
  is std's, undesigned.

## Open

- A collation / locale-aware compare in std.
- Normalisation (NFC/NFD) in std.
- Building text piece by piece efficiently (a builder) — `++` allocates
  each time; the prover may fold chains, but a `list[str]` + `std::str.join`
  is the honest shape today.

## The empty text, and looking for nothing (owner, 2026-10-14)

**An empty text is written `str.empty`** (owner: "(a)"), as `map.empty` is the empty map: `let 'out' :=
str.empty;`. A written `""` is refused, naming it, so an empty text is always meant. **`part`, `at` and
`from` are texts of at least 1 byte** (`str.utf8[min 1 bytes]`): `str.contains`, `starts_with`,
`ends_with` and `find.*` with an empty `part`, `str.split` with an empty `at`, `str.replace` with an empty
`from` are refused (owner: "Error"); an empty one finds nothing anyone means. Whether a text is empty is
`'s' equal_to str.empty` or `str.bytes[lend 's'] equal_to 0`.
