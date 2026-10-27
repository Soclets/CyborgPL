# Input, and range types

> Revised 2026-10-07: code here is in the new syntax; an old spelling in the prose is history. The old → new table is `revision-2026-10-07.md`.

Decided 2026-09-20. The last pillar-level hole: the language had no way to
take a value from outside, and no way to say what that value could be.

## One-ended ranges — `min` and `max` (owner, 2026-09-23)

```
let $pos: int.sign.32[min: 1] = 1;         # at least 1; the ceiling is i32's own
let $score: int.sign.32[max: 100] = 50;    # at most 100
```

`min` and `max` stand for the base type's own limits, so a one-ended
range never writes a number that goes stale when the width changes —
`i8[min: 1]` is 1 to 127 and `i32[min: 1]` is 1 to 2,147,483,647,
from the same line. A position is naturally `i32[min: 1]`, since
indexing is 1-based. `i32` with no bound and `u8[min: 0, max: 255]` are the
plain type written long: harmless, so allowed (owner, 2026-09-23).
Proposed alternatives: `…` and `...`, both out — `...` is the variadic
and the spread (owner spotted it), and the single-character ellipsis
cannot be typed; `*`, which says "something" where `max` says *which*
limit. Owner chose words, as with `x`, `mod`, `and`, `shl`.

## Reading — a boundary in std

```
let $text := std::input.line() otherwise "";      # one line, newline removed; fails with `closed`
let $all  := std::input.all() otherwise "";       # everything until the end
```

Text as it came, `str.utf8`, no interpretation. A number is that plus the
conversion: two lines, two failures, each handled where it happens —

```
let $text := std::input.line() otherwise "";
let $n := to.int.sign.32($text) otherwise 0;
```

(A failing call must be the whole right-hand side, so they cannot nest;
Claude first wrote them nested and the owner caught the `]`.)

## What the prover knows about an outside value

After `to.i32['text']`, `'n'` is any `i32`. **The type is the first
annotation** (owner: "doesn't `i32` already say it?"): `for 'i' in
range[1, 'n']` is provably finite. For a tighter fact — an index into ten
elements, a buffer of `'n'` bytes — something must narrow it. Three ways
were offered: a guardrail `if` (no syntax), a bound on the conversion, or
**range types**. Owner: c.

## Range types — `i32[min: 0, max: 100]`

**Each bound says which it is, `min` or `max`, and a side left out is the
type's own limit** (owner, 2026-10-12: "(a)"): `i32[min: 1]` is "at
least 1". From 2026-09-23 until then the bounds were bare and `min` /
`max` named the type's limits, `i32[1, max]`.

A type with bounds inside it, both ends included, the `range[0, 100]`
shape. A value of that type is *provably* within the bounds, everywhere
it goes.

```
let $n: int.sign.32[min: 0, max: 100] = …;
fn draw [let $x: int.sign.32[min: 0, max: 1023], let $y: int.sign.32[min: 0, max: 767]]:
    …
end of fn draw
rec Percent:
    var value: int.unsign.8[min: 0, max: 100];
end of rec Percent
let $unit: float.bin.64[min: 0.0, max: 1.0] = 0.25;
```

- Bounds are numeric literals of the base type's family (`i32[min: 0.0,
  max: 1.0]` and `f64[min: 0, max: 1]` are errors); low first. Each bound is a value the base type
  can hold: `u8[min: 0, max: 300]` and `u8[min: -1, max: 5]` are errors (owner,
  2026-09-23: "does `u8` hold 300?").
- **Two ways in.** *Assignment* — `let 'n': i32[min: 0, max: 100] = 'raw';` — is a
  check site like any typed assignment: proven by a guardrail `if` (or by
  the value's own range), a runtime check while building, an error in the
  final build. *Conversion* — `to.i32[min: 0, max: 100]['text'] otherwise …` —
  **fails** with `out_of_range` (beside `not_a_number`), so a bad outside
  value is a failure you handle, never a crash; past it, `'n'` is
  `i32[min: 0, max: 100]`.
- the prover carries the bounds through arithmetic: `'n' plus 1` is an
  `i32[min: 1, max: 101]`; storing it back into an `i32[min: 0, max: 100]` is a check
  site; `'xs'['n']` on a 100-element list is proven.
- A range type cannot cross to C (C has no ranges — use the plain type).
  `size.of` is the base type's.

## Oracle

`std::input.line` / `.all` read the process's stdin (`closed` on EOF —
the test harness runs with no stdin, so `examples/input.cy` prints its
fallback). Range types: `examples/ranges.cy`; assignment out of range
is a site error (a "lied" under `no_check`), conversion out of range is
the `out_of_range` failure; ten reject tests. The oracle does not track
arithmetic ranges (`'n' plus 1`) — a value is checked when stored.

## Open

