# Lists and arrays

Decided 2026-09-17.

## Two types

- **`list`** — grows and shrinks. `var.list.int32 'xs' = [1, 2, 3];`
- **`array`** — fixed at the size it was made with. `var.array.int32 'xs' = [1, 2, 3];`

The words carry their everyday meanings: a list is something you add to, an
array is an arrangement in place. The chain narrows to the element type,
which may itself be a chain: `list.str.utf8`, `array.list.int32`.

With `:=`, Precog chooses: a sequence nothing ever appends to becomes an
`array` — known length, provable indexes, placed wherever is fastest. The
programmer only says which at a boundary.

## Literals (decided 2026-09-14)

`[72, 95, 38]` — comma-separated values in a bracket. `[7,]` is a one-element
list (`[7]` is the number seven). `[[1, 2], [3, 4]]` nests.

## Indexing — 1-based

`'xs'[1]` is the first element; `'xs'[0]` does not exist and is an error.
The index is an ordinal, the way people count. This lines up with inclusive
ranges: `loop.temp.range 'i' [:= 1, := list.length[lend 'xs']]` walks the
whole list with no `- 1` anywhere. An out-of-range index is a runtime check
in the default tier, proven away by Precog where it can be.

Assigning to an element (`'xs'[2] = […]`) is **not designed yet**.

## Operations — the type name as the chain's head

```
var 'n' := [list.length[lend 'xs']];
list.append[lend.mut 'xs', 40];
list.remove[lend.mut 'xs', 1];            // by 1-based position
var 'm' := [array.length[lend 'fixed']];
```

They are language operations (the compiler implements them; Precog knows
`array.length` is a constant and `list.length` is the tracked length), spelled
as calls with the ownership verbs: `lend` to look, `lend.mut` to change. No
`std::`, no reserved verb names, no chain hanging off a variable. `list` and
`array` are keywords.

`array` has `length` only; `list` has `length`, `append`, `remove`. Appending
to an `array` is a compile error. A list owns its elements: `append` takes a
fresh value, a `copy`, or a `give` — never a lend.

## Open

- Element assignment.
- Stating a length at a boundary (a list from input) — the range-annotation
  question.
- The empty list: `[]` is not a value; an empty list is only possible in the
  typed form, and needs a spelling.
- Index alongside element in `loop.for` (parked from 2026-09-17).
