# Structs

> Revised 2026-10-07: code here is in the new syntax; an old spelling in the prose is history. The old → new table is `revision-2026-10-07.md`.

Decided 2026-09-18; spelling revised 2026-09-19 (name-then-type fields,
no brackets around field values). A struct is a blueprint: a named shape made of typed
slots. A struct *value* is that shape filled in; each value has its own copy
of every field.

## Declaring the blueprint

```
struct Locker {
    name: str.utf8,
    number: i32,
    capacity: i32,
    occupied: bool
}
```

- Top level, beside `fn`; `pub struct` exports it (`kits.md`). The name is
  bare (a type, like `i32`); **field names are bare too** (decided
  2026-09-19, after a cold read caught `'number': i32` in the declaration
  but `'l'.number` at access): a field is a path *into* a struct, not a
  variable you hold. The quotes go on the whole — `'l'` — and the route
  into it (`.number`, like `[2]` into a list) is bare. `name: type`,
  the type after the name like every other site.
- No declaration word on a field: a struct always *owns* its fields, so there is nothing
  to choose (a parameter's `lent` chooses; a field cannot).
- No `;` after a field: fields are not statements. Fields are separated by `,`, and a
  `,` after the last is allowed (owner, 2026-10-14, as everywhere items sit in braces:
  enum cases, `layout.C` structs and enums, precedence levels).
- `!mut` is allowed on a field — `!mut number: i32` — under the same
  rule as `!mut` on a variable: a verified contract, allowed, not standard.
- A field may be any type, including another struct or a list.

## Making a value — by field name

```
let 'mine' := Locker[name = "field kit", number = 7,
                 capacity = 12, occupied = false];
```

- The struct name, bare, before `[ ]`, like a call — this bracket *builds*
  the value, so it stays; inside, each field by name with `=` (the blueprint
  fixed the types, so it is the typed form).
- **Every field must be given** — there are no zero values. Order is free.
- A field's value is a bare expression: `name = "locker " ++ 'n'`,
  `tags = [1, 2]`.

## Reading and writing a field

```
let 'n' := 'mine'.number;           # read
change 'mine'.number = 8;                  # write
```

A quoted name followed by `.` is field access — a new shape, unambiguous
because nothing else follows a quoted name with a dot. Writing obeys the
same rules as assignment: the struct must be assignable (not `!mut`, not a
shared `lent`), the field must not be `!mut`.

## Ownership

**A struct is always an owned value** (decided 2026-09-18), even when every
field is plain: a struct owns its fields, so `give` / `copy` / `lend` /
`lend.mut` always apply to the whole. One rule, and adding a `str` field
later never changes how the struct is passed around. the prover elides the
copies it can prove unnecessary. For the parts:

- A plain-value field (`'mine'.number`) copies freely, like any plain value.
- An owned field needs a verb: `copy 'mine'.name`, `lend 'mine'.name`.
- **`give 'mine'.name` is allowed and consumes the whole variable.** After
  it, `'mine'` is gone — every field with it — until reassigned. There is no
  half-empty struct (Rust's partial move is not reproduced). This keeps the
  one real use — pulling a large field out without copying — at the cost of
  the container.
