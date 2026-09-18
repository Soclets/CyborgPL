# Structs

Decided 2026-09-18. A struct is a blueprint: a named shape made of typed
slots. A struct *value* is that shape filled in; each value has its own copy
of every field.

## Declaring the blueprint

```
struct Locker {
    str.utf8 'name'
    int32 'number'
    int32 'capacity'
    bool 'occupied'
}
```

- Top level, beside `func`. The name is bare (a type, like `int32`); the
  fields are variables, so quoted; type first, then name.
- No `var` on a field: a struct always *owns* its fields, so there is nothing
  to choose (a parameter's `var`/`lent` chooses; a field cannot).
- No `;` after a field: fields are not statements.
- `immut.` is allowed on a field — `immut.int32 'number'` — under the same
  rule as `var.immut`: a verified contract, allowed, not standard.
- A field may be any type, including another struct or a list.

## Making a value — by field name

```
var 'mine' := [Locker['name' = ["field kit"], 'number' = [7],
                      'capacity' = [12], 'occupied' = [false]]];
```

- The struct name, bare, before `[ ]`, like a call; inside, each field by
  name with `=` (the blueprint fixed the types, so it is the typed form).
- **Every field must be given** — there are no zero values. Order is free.
- Every field value wears its own `[ ]` — the value rule, no exception.

## Reading and writing a field

```
var 'n' := ['mine'.number];         # read
'mine'.number = [8];                # write
```

A quoted name followed by `.` is field access — a new shape, unambiguous
because nothing else follows a quoted name with a dot. Writing obeys the
same rules as assignment: the struct must be assignable (not `immut`, not a
shared `lent`), the field must not be `immut`.

## Ownership

A struct holding an owned field (a `str`, a list, another such struct) is an
owned value; `give` / `copy` / `lend` / `lend.mut` apply to the whole as
usual. For the parts:

- A plain-value field (`'mine'.number`) copies freely, like any plain value.
- An owned field needs a verb: `copy 'mine'.name`, `lend 'mine'.name`.
- **`give 'mine'.name` is allowed and consumes the whole variable.** After
  it, `'mine'` is gone — every field with it — until reassigned. There is no
  half-empty struct (Rust's partial move is not reproduced). This keeps the
  one real use — pulling a large field out without copying — at the cost of
  the container.
