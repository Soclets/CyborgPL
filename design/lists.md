# Lists and arrays

> Revised 2026-10-07: code here is in the new syntax; an old spelling in the prose is history. The old → new table is `revision-2026-10-07.md`.

Decided 2026-09-17.

## Two types

- **`list`** — grows and shrinks. `let 'xs': list[i32] = [1, 2, 3];`
- **`array`** — fixed at the size it was made with. `let 'xs': array[i32] = [1, 2, 3];`

The words carry their everyday meanings: a list is something you add to, an
array is an arrangement in place. **The element type is a type parameter in
brackets** (decided 2026-09-19, after a cold read noted that a dot chain
cannot carry two parameters — `map.str.utf8.i32` has no unambiguous
reading): `list[i32]`, `list[str.utf8]`, `array[list[i32]]`. The rule
for types is now: `.` picks a *fixed variant* (`str.utf8`, `char.grapheme`),
`[ ]` supplies a *type* — Scala's spelling, and the `[ ]`-builds rule applied
to types. Until then the element type rode on the chain (`list.i32`).

With `:=`, the prover chooses: a sequence nothing ever appends to becomes an
`array` — known length, provable indexes, placed wherever is fastest. The
programmer only says which at a boundary.

## Slicing — `list.slice`, `array.slice` (owner, 2026-09-23)

```
let $mid := std::ls.slice(lend $xs, from: pos 2, to: pos 4);      # elements 2, 3 and 4
let $head := arr.slice(lend $fixed, from: pos 1, to: pos 2);
let $none := std::ls.slice(lend $xs, from: pos 3, to: pos 2);     # empty: last is first minus 1
```

**A slice owns its elements**, like any list: the copy is the meaning.
the prover elides it wherever nobody can tell — a slice that is read and
dropped while the original is untouched becomes a pointer and a length,
the same machine code a borrowed view would give. The copy appears only
where it must: when the slice outlives the original, or when either is
written to. Proposed alternative: a borrowed view, as Rust's `&[T]` and
Go's slices are — faster always, but a third category beside owned and
lent, needing its own type everywhere and refusing programs that escape
it. The owner asked whether the prover could help, and it is what decides
this: **(a) always compiles and sometimes copies; (b) never copies and
sometimes refuses.**

1-based and inclusive, like `[i]` and `range[…]`, with the positions
written bare, as every index is. A **check site**, not a failure, because
it is a language operation — `std::str.slice.grapheme` fails instead
because everything in std fails. `array` gets it too (owner): slicing
reads, so the reason an array has no `append` does not apply.

**Either way, a slice is a `list[T]`** (owner, 2026-09-23, as part of
the same answer): `array.slice` gives a list too, and the prover makes it an
array wherever it proves the length, which it usually can, the way any
list nobody appends to becomes one. So `list.append` onto a slice of an
array is allowed. (Written down late: the answer was given with the
question and left out of the first record. Compiler claude asked again,
recommending an array from an array.)

## Between the two — `to.array[…]`, `to.list[…]` (owner, 2026-09-23)

```
let $a := to.arr(give $l);     # a list's length frozen: it cannot grow now
let $m := to.ls(copy $a);      # an array that can grow; 'a' is still here
```

A `list` and an `array` are different types, and nothing converts by
itself (2026-09-19): a list given to an array variable, passed for an
array parameter or returned as an array is refused, and the other way
round too. A literal and a `:=` collection are neither yet, so they go
either way, as before. The element type does not change, so it is not
written (`to.array[i32][…]` is refused). `give` hands the elements
over without a copy, and `copy` keeps the original. **A bare name is
refused** (owner: a): `to.array['l']` says neither, and a list always
says `give`, `copy` or `lend`, so a copy is never made quietly. The
alternative was a quiet copy. A `lend` is refused too (owner: "aren't you
supposed to use another ownership word?"): a lend only lets you look,
and a conversion takes, so the word is `copy`. The owner
asked "don't we usually have converters for these?", which was the
2026-09-19 rule, so the only new part was the spelling.

## Layout (owner, 2026-09-22)

A **`list` is 24 bytes** — a pointer, a length and a capacity — because
it is the thing that grows; an **`array` is 16** — a pointer and a
length — because its length never changes; a `str` is 16 for the same
reason (`types.md`). Each type is exactly as big as it needs to be,
rather than one handle shape for all three (proposed, and declined: it
would put 8 unused bytes on every text).

the prover's part is the type itself, as it always was: a sequence nothing
ever appends to becomes an `array`, so the capacity is gone before
layout is asked. And **a `list` whose final size it can prove** — it
grows to exactly N and stops — **needs no capacity field either**: the
capacity is a constant the compiler holds, the allocation is fixed, and
the handle is 16 bytes. Same shape as the text builder's hidden
capacity in `types.md`.

## Literals (decided 2026-09-14)

`[72, 95, 38]` — comma-separated values in a bracket. `[7,]` is a one-element
list (`[7]` is the number seven). `[[1, 2], [3, 4]]` nests.

A bare number in a literal takes its family from the other items (owner,
2026-09-30: a): `[1, 2.5]` is a list of bins, the `1` a bin like the `2.5`
beside it; a list still holds one element type. With `:=`, the prover picks
list or array and the width, as ever. Proposed alternative: a mixed
literal needs a declared type or `1.0`.

## Indexing — `'xs'[2]`, 1-based

`'xs'[1]` is the first element; `'xs'[0]` does not exist and is an
error. The position is written bare (owner, 2026-09-27: "Remove `pos`").
From 2026-09-17 every index said `pos` (`'xs'[pos: 2]`), the owner's own
proposal, so a reader never wondered whether `2` meant "the second" or
"the value 2", and since 2026-09-21 it also told a list read (`'xs'[pos: 2]`)
from a call of a function value (`'f'[2]`). Now both are `'x'[2]`, and
what the variable holds decides: a list is read, a function is called.
The compiler always knows which, and the owner's view is that a reader
has names and comments. A list read standing alone as a statement does
nothing and is refused. `pos` is no longer a keyword. The position is an
ordinal, the way people count,
which lines up with inclusive ranges:
`for 'i' [from: 1, to: list.length[lend 'xs'], by: 1]` walks the whole list
with no `- 1` anywhere. An out-of-range position is a runtime check in the
default tier, proven away by the prover where it can be.

**Element assignment** (decided 2026-09-17) is the same shape on the left of `=`:

```
change $xs[pos: 2] to 5;
```

The list must be assignable (not `!mut`, not a shared `lent`); the position
is range-checked; the value obeys the ownership verbs (an owned value is
`give`n or `copy`d in). Works on `array` too — writing into a slot is not
growth.

Selecting by *value* (`'xs'[value-is 2]`) was considered and dropped: it is a
search, and a search is a loop the reader should see —

```
for perm $i [from: 1, to: std::ls.len(lend $xs), by: 1]:
    if $xs[pos: $i] eq 2:
        break;
    end of if
end of for
change $xs[pos: $i] to 5;
```

— which the `perm` counter makes three plain lines.

**Changing elements in a walk** (owner, 2026-09-30). Over `lend.mut 'xs'`,
the loop's element is the list's own: changing it changes the list. Over
`lend`, it cannot be changed.

```
for $x in lend.mut $prices:
    change $x to $x times 2;
end of for
```

Inside such a loop, the list itself may be read (`'xs'[1]`,
`list.length[lend 'xs']`) only after the element's last use in that trip,
and the read sees the values as changed so far. A read with the element
still to be used after it is refused. This is Rust's borrow rule applied
trip by trip: a borrow lasts until its last use. Rust's `iter_mut` is
stricter and refuses any read of the list inside the loop. Changing,
growing or giving away the list inside stays refused for the whole loop.

## Operations — the type name as the chain's head

```
let $n := std::ls.len(lend $xs);
std::ls.append(lend.mut $xs, value: 40);
std::ls.remove(lend.mut $xs, pos: 1);            # by 1-based position; the element is dropped
let $first := std::ls.remove(lend.mut $xs, pos: 1);   # or handed back (decided 2026-09-21)
run(std::ls.remove(lend.mut $queue, pos: 1));    # straight into a call: moved, never copied
let $m := arr.len(lend $fixed);
```

They are language operations (the compiler implements them; the prover knows
`array.length` is a constant and `list.length` is the tracked length), spelled
as calls with the ownership verbs: `lend` to look, `lend.mut` to change. No
`std::`, no reserved verb names, no chain hanging off a variable. `list` and
`array` are keywords.

**A length is a `u64`** (decided 2026-09-21): a count is never negative
and never too big for 64 bits. `let 'n' := list.length[lend 'xs'];` makes
`'n'` a `u64`; `let 'm': i32 = list.length[lend 'xs'];` is a narrowing
check site — the prover proves it wherever it can see the list (a literal, a
guarded `if`), and the tiers handle the rest. The written type is the
promise; the final build stores a length it has proven small in whatever
is fastest.

## A length bound — `list[T][lo, hi]`

```
var $ys: std::ls[int.sign.32][min: 0, max: 1000] = read_numbers() otherwise std::ls.empty;
let $n := std::ls.len(lend $ys);      # a u64[min: 0, max: 1000]
let $m: int.sign.32 := $n;                   # proven by the bound; no check
```

Decided 2026-09-21: the thing, then its bounds — the same shape as
`i32[min: 0, max: 100]`. It is a fact about the data, not about one spot in the
code, so everything downstream knows it: the list's length is a
`u64[lo, hi]`, and narrowing it is proven. Every change to the list
(a construction, an assignment, `append`, `remove`) is checked against the
bound — a check site like writing 101 into an `i32[min: 0, max: 100]`: the prover
proves it where it can see the list, the tiers handle the rest. The low
bound is 0 or more; an `array` takes no bound (its one length is already
known). Inside a call the parameter is written the same way:
`lent 'ys': list[i32][min: 0, max: 1000]`.

**`list.remove` hands the removed element back** (decided 2026-09-21): the
list stops owning it, so the caller does — or drops it by not taking it.
This is the one way to move an owned element *out* of a list without a
copy. `give 'xs'[i]` stays refused: it would leave a hole (a slot
with nothing in it, which every later access would have to check for —
a cost on every list in the program), and shrinking-by-give would be a
second spelling of `list.remove`. Positions that must stay put are the
element type's business: `enum Slot { empty, job: Job }`. `list.append`
hands nothing back and cannot be used as a value.

`array` has `length` only; `list` has `length`, `append`, `remove`. Appending
to an `array` is a compile error. A list owns its elements: `append` takes a
fresh value, a `copy`, or a `give` — never a lend.

## The empty list (decided 2026-09-17)

```
let $xs: std::ls[int.sign.32] := [];             # typed: the type says list, [] is empty
let $ys := std::ls.empty;                # best-type: the value says list; Precog picks the
                                       # element type from what is appended later
```

`[]` is a value only where a declared type says it is a list; anywhere else
it is still "not a value". With `:=` nothing on the left says "list", so the
value has to: `list.empty`. An empty `array` could never be filled, so it
is an error.

## The position beside the item: `for 'i', 'x' in lend 'xs'` (owner, 2026-09-29: a)

```
for $i, $x in lend $xs:
    std::print.shortest($i, ". ", $x, newln);      # 1. ann  2. bob  3. cy
end of for
```

The position first, then the item, the order of `'xs'['i']`. The position
counts 1, 2, 3… and is a `u64`, like every position. A range needs
none, since its counter is already the position, so `for 'i', 'x' in
range[…]` is refused. With `perm`, both outlive the loop (the oracle's
reading, not asked). Proposed alternative: leave it, looping over
`range[1, list.length[…]]` and reading `'xs'['i']`, one extra line.
`examples/for_position.cy`.

## Open

## A list of n copies — `list.fill` (owner, 2026-09-30)

```
var $scores = std::ls.fill(value: 0, count: 10);              # ten 0s
let $flags: std::ls[bool] := std::ls.fill(value: false, count: $n);   # 'n' falses
```

The value, then the count, as `simd.fill[7]` fills every lane. A count
below 0 is a check site. A **`#strict` file labels the count**,
`list.fill[0, count: 10]` (owner: "count"), since `list.fill[0, 10]`
could be ten 0s or zero 10s; elsewhere `count` is refused, as `pos` is.
With it, kayo can ask for memory already zeroed (C's `calloc`) where the
value is `false` or `0`, instead of growing a list one append at a time.
Proposed alternatives: `list.repeat[…]`, Rust's `[0; 10]`; for strict,
both labelled (`value 0, count: 10`) or no change. `array.fill` was
mentioned, not asked. `examples/list_fill.cy`, `list_fill_strict.cy`.

