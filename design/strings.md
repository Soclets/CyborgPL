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

## Prints share one buffer, flushed by kayo (owner, 2026-10-23; not built)

kayo keeps one output buffer across prints and sends it whenever the order could matter: at the end of the program,
before reading input, before a C call, before a print to the other stream, and when a check fails (design claude's
list). The program writes no flush; the report notes where prints were grouped. Owner: "yes, and there should also be
a manual way", then "both, bro": (a) a flush the program calls, sending what kayo holds now, and (b) a writer the
program makes, with its own buffer and size, passed to functions and flushed when wanted; kayo refuses a writer that
goes away unflushed. Spellings open. Its size is in bytes, the unit written: `size: 4096 bytes` (owner: "Just `bytes`
for now", over also allowing `4 KiB`).

```
var $out := std::print.writer(size: 4096 bytes);   # std::print.writer.stderr(…) for stderr
std::print.shortest(lend.mut $out, "line ", $i, newln);  # a first argument `lend.mut` of a writer: into it
std::print.flush(lend.mut $out);                       # left out, kayo refuses: $out goes away holding text
std::print.flush();                                    # what kayo's own buffer holds, now
```

A print never changes what it prints, so a mutable lend can only be where the text goes; there is no `.to` (owner,
2026-10-23, "yea, record it"). The type is `std::print.writer`, and every print form takes a writer the same way
(design claude's readings).

**A print that does not fit** (owner, 2026-10-24): a writer sends what it holds and keeps the new print whole;
`std::print.writer.whole(…)` says so, the default; `std::print.writer.split(…)` fills to exactly its size and may
split a print, for raw throughput. A print longer than a whole writer goes out alone (design claude's reading).

**A failed write stops the program** (owner, 2026-10-23: "default to stop, handle with `otherwise`?", then "yea,
record it"). A print with `otherwise` flushes there, so its handler covers all printed so far:
`std::print.useful("total: ", $total, newln) otherwise { write_failed { std::exit(code: 3); } };`. A writer fails at a
print that fills its buffer or at its flush; a flush kayo put in itself stops the program. `write_failed` is design
claude's name. With no `otherwise`, the stop is exit 1 and `error[write_failed]` on stderr (owner, 2026-10-24: "(a)"). Today each print sends its own 512-byte stack buffer when it ends.

## `std::print.written` — a literal as it was typed (owner, 2026-10-17; not built)

A third form beside `print.stored` and `print.useful`: each value printed as its literal was typed in the source,
even past what the type holds (`let 'x': f16 = 0.123456789;` prints `0.123456789`; stored `0.12347412109375`,
useful `0.1235`). Precog proves at compile time that every value printed is a typed literal; where it cannot,
the print says what to do, `std::print.written['total', newln] otherwise std::print.useful;`, or it is refused.
Decided when building: no source text is carried at run time.

## `std::print.useful.auto_newline` — the line ended for you (owner, 2026-09-30)

```
std::print.shortest.auto_newline("Hello, World!");
```

`print.useful` with a newline after the values, the same as writing `\n`
last. The name says what it adds, where `println`'s `ln` hides it. It is
the first chain word with `_` in it. The owner wrote it "for fun", after
comparing the Hello World with Java's; `std::rs.println` (2026-09-19,
removed a day later) had done the same as a borrowed Rust print. Only this
form is decided: `print` and `print.format` have no `auto_newline`.
`examples/print_auto_newline.cy`.

## `std::print.weird` — C++'s `cout <<`, with arrows (owner, 2026-10-01)

```
std::print.arrows.shortest <- "tenth is " <- $tenth <- ", n is " <- $n <- newln;
std::print.arrows.stored <- "stored: " <- $tenth <- newln;
```

A print in the shape of C++'s `std::cout << …` (owner: "`std::print.weird`
seems like a good name, maybe use `->` arrows"). Each value follows a
**`<--`**, pointing into the output, as C++'s `<<` points into `cout`
and as `<-` points into a variable in `inline.ASM` (owner: "`<-`", over
`-->`, which would point away from the screen — CyborgPL's arrows point
where the value goes). It takes no `[ ]`, and at least one value. **`.weird.stored`**
shows the stored form, as `print.stored`; **`.weird.useful`** the
useful form, as `print.useful` (a bare `.weird` said "stored" until
2026-10-02, when every print came to say which) (owner: "what if we add a way to use
`.useful` with `.weird`?", then "yea"). The strongest modifier first:
`.weird` changes the whole call's shape, `.useful` only how values
look, so `print.useful.weird` is refused. The first borrowed-language
print since `std::c.printf` / `std::rs.println` were removed
(2026-09-20). `examples/print_weird.cy`; reject tests
print_weird_brackets, print_weird_order.

## `std::print.format` — the formatting print, with `{}` holes (2026-09-22; holes since 2026-10-01)

```
std::print.format("{}: {} at ${places: 2}, 50% off{newln}", $name, $count, $price);
std::print.format("0x{base: hex} 0b{base: bin, zeros, width: 8 cols} [{width: 6 cols}] [{sign}]{newln}", 255, 10, 42, 5);
std::print.format("{2} vs {1}, and {2} wins{newln}", "a", "b");
std::print.format.stderr("error: {}{newln}", $msg);
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
- **Shapes are words** after the hole's number, if any, in any order (owner, 2026-10-25, over Rust's `{:>8.2}`,
  design claude's pick of 2026-10-01, now an old spelling refused naming its words). **A shape with several choices
  is a label and its value; one that is only on or not is a bare word** (owner: "It was `exclude zero`, because
  there's nothing except that"):
  - `align: left` / `right` / `center`: a number sits right and a text left unless it says;
  - `width: 8 cols`: the least the value takes, **in a named unit** (owner: "width: 8 what?"): `cols` screen
    columns (`1 col`, `8 cols`; a Chinese, Japanese or Korean character and most emoji 2, a combining mark 0, an
    East Asian Ambiguous one like `°` 1), `cols.east_asian` (that last kind 2, for terminals set to those
    languages), `graphemes`, `points` or `bytes`. Unicode 17.0.0's `EastAsianWidth.txt` and `emoji-data.txt` say
    which, as terminals' `wcwidth` does; a program cannot know its terminal, so asking it is the writer's own code;
  - `` fill: `*` ``: the padding, a char literal (a space unless written), one of the width's units wide;
  - `places: 2`: digits after the point, for numbers only;
  - `base: bin` / `oct` / `hex` / `hex.upper`; `form: stored` (the stored form, `0.1000000000000000055…` for an
    `f64` `0.1`) / `form: science` (`1.23e3`);
  - `zeros` (pad with zeros after the sign, `-0005`) and `sign` (always show it, `+5`).

  ```
  std::print.format("  {align: left, width: 10 cols} x{width: 2 cols}  {width: 8 cols, places: 2}\n", …);
  std::print.format("{2, base: hex.upper, zeros, width: 4 cols} {1}\n", "id", 255);   # 00FF id
  ```

  There is no `#`: type the prefix, `0x{base: hex}`. `examples/print_format_width.cy`.
- A negative whole number in a base is its sign and digits, `{base: bin}` of -5 being `-101` (owner, 2026-10-24: "(b)");
  the stored bits are asked for by name, `to.u8.bits($b)`.
  `{places: 2}` of a `d64` is exact, from its decimal digits, half to even.
- **Lists, structs and maps** fill a plain `{}` or `{form: stored}`, as `print.useful`
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
`std::rs.println` of 2026-09-19 stay removed. `examples/print_format.cy`;
reject tests `print_format_*`.

## A length bound names its unit (decided 2026-09-22)

```
let $name: str.utf8[min: 1, max: 40 %clusters] := std::input.line() otherwise "";
fn greet [lent $who: str.utf8[min: 1, max: 10 %clusters]]:
    …
end of fn greet
let $field: str.utf8[min: 0, max: 255 bytes] := …;
let $code: str.utf8[min: 4, max: 4 %points] := …;
```

The thing, then its bounds, as `i32[min: 0, max: 100]` and `list[T][lo, hi]` —
and, because a text has three lengths and no single one, the bound
**says which it counts**: `bytes`, `points`, or `graphemes`, after the
high bound. `str.utf8[0, 20]` with no unit is a compile error. Every
change to a bounded text (assignment, `++`) is a check site against the
bound; a conversion into one (`to.str.utf8[min: 0, max: 3 bytes][…]`) fails
`out_of_range` like any conversion into a range type. the prover carries
the length interval per unit already. Proposed: bytes always, graphemes
always, or say which; owner chose "say which".

## No single length

`"é🍎"` is 5 bytes, 2 code points, 2 graphemes; a family emoji is 18
bytes, 5 points, 1 grapheme. The language has the three units already
(`char.byte` / `.point` / `.grapheme`), so it has three counts and no
`str.length` — the thing that makes other languages cut emoji in half:

```
std::str.bytes(lend $s)        # what the OS and C see; O(1)
std::str.points(lend $s)       # Unicode scalar values
std::str.clusters(lend $s)    # what a person counts as one character; O(n)
```

`str.length` is a compile error naming the three.

## Walking says the unit too

```
for $g in std::string.each.grapheme_cluster(lend $s):
    …
end of for     # 'g': char.grapheme
for $p in std::string.each.code_point(lend $s):
    …
end of for        # 'p': char.point
for $b in std::string.each.byte(lend $s):
    …
end of for         # 'b': u8
```

A bare `for 'c' in lend 's'` is an error: a text is not a list of anything
until you say what. `each.<unit>` yields the units without building a
list (the prover lowers it to a loop over the bytes; the oracle builds the
list).

## No `[i]` on a text

`'s'[3]` is a compile error — the 3rd byte or the 3rd grapheme? The
positional forms live in std with the unit spelled and a failure for out
of range: `std::str.at.grapheme[lend 's', pos: 3] otherwise …`,
`std::str.slice.grapheme[lend 's', from: pos 1, to: pos 5] otherwise …`. Most "the 3rd
character" code is a `split` or a `find` in disguise.

## The toolbox is std's

The language owns only what the compiler must *know* — the units and how
to walk them. Everything else is ordinary library code, and case tables
change with every Unicode release, so they belong in something updatable:

```
std::str.split(lend $s, at: " ")          -> std::ls[str.utf8]
std::str.join(lend $parts, between: "_")
std::str.contains / starts_with / ends_with(lend $s, "x")   -> bool
std::str.trim(lend $s)
std::str.upper.ascii / .simple / .full(lend $s)          (and std::str.lower.…; `.full(…, lang: std::casing.tr)`)
std::str.replace(lend $s, from: "a", to: "b")
std::str.find.cluster(lend $s, part: "x")  -> int.sign.32 or fail[missing]   (1-based grapheme position)
std::str.at.<unit>(lend $s, pos: 3)        -> that unit, or fail[out_of_range]
std::str.slice.cluster(lend $s, from: pos 1, to: pos 5) -> str, or fail[out_of_range]
std::str.slice.point(lend $s, from: pos 1, before: pos 5)  -> str, or fail[out_of_range]
std::str.slice.byte(lend $s, from: pos 1, to: pos 5)     -> std::ls[int.unsign.8], or fail[out_of_range]
```

**A case change says its mapping** (owner, 2026-10-20): `str.upper.ascii` changes a to z only (`café` → `CAFé`),
`.simple` is one character for one (`ß` stays `ß`), `.full` is Unicode's (`straße` → `STRASSE`; the length may change
in every unit, `ŉ` → `ʼN`); the same for `str.lower`. A bare `str.upper` is refused naming them. The full mapping takes
a language's own rules with `lang:`, a value of std's enum **`casing`**: `tr`, `az`, `lt`, or `root` for every language
without rules of its own (Unicode's name for its language-independent rules): `str.upper.full[lend 's', lang: tr]`
gives `İSTANBUL`. std's enums let their values drop the enum's name, so `lang: tr` is `lang: casing.tr`. A language
tag from the system becomes one with `to.casing['tag']`, its first part deciding (`"tr-TR"` → `tr`, `"en-GB"` →
`root`), never failing. `examples/case_mappings.cy`.

**A byte slice gives bytes, not text** (owner, 2026-10-20: "(a)"). A cut by bytes can land inside a character
(`"é"` is 2 bytes), so `str.slice.byte` hands back a `list[u8]`, as `str.at.byte` hands back a `u8`; text
is made from it with `to.str.utf8[…] otherwise …`, which fails `not_utf8` on a cut character, so a broken
character is caught where it happens and never becomes a `str`. It is for where text meets something that counts
bytes (a field's byte limit, a C buffer, an offset from a file or protocol); ordinary code slices by grapheme.

The oracle emulates these so examples run (`examples/text.cy`); std's
real versions will be CyborgPL.

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
`from` are texts of at least 1 byte** (`str.utf8[min: 1 bytes]`): `str.contains`, `starts_with`,
`ends_with` and `find.*` with an empty `part`, `str.split` with an empty `at`, `str.replace` with an empty
`from` are refused (owner: "Error"); an empty one finds nothing anyone means. Whether a text is empty is
`'s' equal_to str.empty` or `str.bytes[lend 's'] equal_to 0`.
