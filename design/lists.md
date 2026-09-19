# Lists and arrays

Decided 2026-09-17.

## Two types

- **`list`** — grows and shrinks. `let 'xs': list[int32] = [1, 2, 3];`
- **`array`** — fixed at the size it was made with. `let 'xs': array[int32] = [1, 2, 3];`

The words carry their everyday meanings: a list is something you add to, an
array is an arrangement in place. **The element type is a type parameter in
brackets** (decided 2026-09-19, after a cold read noted that a dot chain
cannot carry two parameters — `map.str.utf8.int32` has no unambiguous
reading): `list[int32]`, `list[str.utf8]`, `array[list[int32]]`. The rule
for types is now: `.` picks a *fixed variant* (`str.utf8`, `char.grapheme`),
`[ ]` supplies a *type* — Scala's spelling, and the `[ ]`-builds rule applied
to types. Until then the element type rode on the chain (`list.int32`).

With `:=`, Precog chooses: a sequence nothing ever appends to becomes an
`array` — known length, provable indexes, placed wherever is fastest. The
programmer only says which at a boundary.

## Literals (decided 2026-09-14)

`[72, 95, 38]` — comma-separated values in a bracket. `[7,]` is a one-element
list (`[7]` is the number seven). `[[1, 2], [3, 4]]` nests.

## Indexing — `pos`, 1-based

`'xs'[pos 1]` is the first element; `'xs'[pos 0]` does not exist and is an
error. `pos` is mandatory on every index (decided 2026-09-17): it says the
number is a *position*, so a reader never wonders whether `2` means "the
second" or "the value 2". The position is an ordinal, the way people count,
which lines up with inclusive ranges:
`for 'i' in range[1, list.length[lend 'xs']]` walks the whole list
with no `- 1` anywhere. An out-of-range position is a runtime check in the
default tier, proven away by Precog where it can be.

**Element assignment** (decided 2026-09-17) is the same shape on the left of `=`:

```
'xs'[pos 2] = 5;
```

The list must be assignable (not `immut`, not a shared `lent`); the position
is range-checked; the value obeys the ownership verbs (an owned value is
`give`n or `copy`d in). Works on `array` too — writing into a slot is not
growth.

Selecting by *value* (`'xs'[value-is 2]`) was considered and dropped: it is a
search, and a search is a loop the reader should see —

```
for perm 'i' in range[1, list.length[lend 'xs']] {
    if 'xs'[pos 'i'] == 2 { break; }
}
'xs'[pos 'i'] = 5;
```

— which the `perm` counter makes three plain lines.

## Operations — the type name as the chain's head

```
let 'n' := list.length[lend 'xs'];
list.append[lend.mut 'xs', 40];
list.remove[lend.mut 'xs', 1];            # by 1-based position
let 'm' := array.length[lend 'fixed'];
```

They are language operations (the compiler implements them; Precog knows
`array.length` is a constant and `list.length` is the tracked length), spelled
as calls with the ownership verbs: `lend` to look, `lend.mut` to change. No
`std::`, no reserved verb names, no chain hanging off a variable. `list` and
`array` are keywords.

`array` has `length` only; `list` has `length`, `append`, `remove`. Appending
to an `array` is a compile error. A list owns its elements: `append` takes a
fresh value, a `copy`, or a `give` — never a lend.

## The empty list (decided 2026-09-17)

```
let 'xs': list[int32] = [];             # typed: the type says list, [] is empty
let 'ys' := list.empty;                # best-type: the value says list; Precog picks the
                                       # element type from what is appended later
```

`[]` is a value only where a declared type says it is a list; anywhere else
it is still "not a value". With `:=` nothing on the left says "list", so the
value has to: `list.empty`. An empty `array` could never be filled, so it
is an error.

## Open

- Stating a length at a boundary (a list from input) — parked with input,
  2026-09-17; it is the range-annotation question.
- Index alongside element in `for` (parked from 2026-09-17).
