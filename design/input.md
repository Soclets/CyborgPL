# Input, and range types

Decided 2026-09-20. The last pillar-level hole: the language had no way to
take a value from outside, and no way to say what that value could be.

## One-ended ranges — `min` and `max` (owner, 2026-09-23)

```
'pos': int32[1, max] = 1;         # at least 1; the ceiling is int32's own
'score': int32[min, 100] = 50;    # at most 100
```

`min` and `max` stand for the base type's own limits, so a one-ended
range never writes a number that goes stale when the width changes —
`int8[1, max]` is 1 to 127 and `int32[1, max]` is 1 to 2,147,483,647,
from the same line. A position is naturally `int32[1, max]`, since
indexing is 1-based. `int32[min, max]` and `uint8[0, 255]` are the
plain type written long: harmless, so allowed (owner, 2026-09-23).
Proposed alternatives: `…` and `...`, both out — `...` is the variadic
and the spread (owner spotted it), and the single-character ellipsis
cannot be typed; `*`, which says "something" where `max` says *which*
limit. Owner chose words, as with `x`, `mod`, `and`, `shl`.

## Reading — a boundary in std

```
'text' := std::input.line[] otherwise "";      # one line, newline removed; fails with `closed`
'all'  := std::input.all[] otherwise "";       # everything until the end
```

Text as it came, `str.utf8`, no interpretation. A number is that plus the
conversion: two lines, two failures, each handled where it happens —

```
'text' := std::input.line[] otherwise "";
'n' := to.int32['text'] otherwise 0;
```

(A failing call must be the whole right-hand side, so they cannot nest;
Claude first wrote them nested and the owner caught the `]`.)

## What Precog knows about an outside value

After `to.int32['text']`, `'n'` is any `int32`. **The type is the first
annotation** (owner: "doesn't `int32` already say it?"): `for 'i' in
range[1, 'n']` is provably finite. For a tighter fact — an index into ten
elements, a buffer of `'n'` bytes — something must narrow it. Three ways
were offered: a guardrail `if` (no syntax), a bound on the conversion, or
**range types**. Owner: c.

## Range types — `int32[0, 100]`

A type with bounds inside it, both ends included, the `range[0, 100]`
shape. A value of that type is *provably* within the bounds, everywhere
it goes.

```
'n': int32[0, 100] = …;
fn draw ['x': int32[0, 1023], 'y': int32[0, 767]] { … }
struct Percent { value: uint8[0, 100] }
'unit': bin64[0.0, 1.0] = 0.25;
```

- Bounds are numeric literals of the base type's family (`int32[0.0,
  1.0]` and `bin64[0, 1]` are errors); low first. Each bound is a value the base type
  can hold: `uint8[0, 300]` and `uint8[-1, 5]` are errors (owner,
  2026-09-23: "does `uint8` hold 300?").
- **Two ways in.** *Assignment* — `'n': int32[0, 100] = 'raw';` — is a
  check site like any typed assignment: proven by a guardrail `if` (or by
  the value's own range), a runtime check while building, an error in the
  final build. *Conversion* — `to.int32[0, 100]['text'] otherwise …` —
  **fails** with `out_of_range` (beside `not_a_number`), so a bad outside
  value is a failure you handle, never a crash; past it, `'n'` is
  `int32[0, 100]`.
- Precog carries the bounds through arithmetic: `'n' + 1` is an
  `int32[1, 101]`; storing it back into an `int32[0, 100]` is a check
  site; `'xs'[pos 'n']` on a 100-element list is proven.
- A range type cannot cross to C (C has no ranges — use the plain type).
  `size.of` is the base type's.

## Oracle

`std::input.line` / `.all` read the process's stdin (`closed` on EOF —
the test harness runs with no stdin, so `examples/input.cpls` prints its
fallback). Range types: `examples/ranges.cpls`; assignment out of range
is a site error (a "lied" under `nocheck`), conversion out of range is
the `out_of_range` failure; ten reject tests. The oracle does not track
arithmetic ranges (`'n' + 1`) — a value is checked when stored.

## Open

