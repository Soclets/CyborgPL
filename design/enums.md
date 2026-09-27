# Enums — one of a few named cases

Decided 2026-09-20. Owner asked "what are enums?" — a type that means *one
of these few options*, like `bool` with your own names, where an option may
carry a detail. A function's `fail[…]` list has this shape already
(`failures.md`); `enum` is the same idea as a type you can hold.

## Declaring

```
enum Status {
    empty
    claimed: str.utf8       # this case carries who claimed it
    locked
}
```

Top level beside `struct`; `pub` works the same. The name is bare (a type);
case names are bare (paths into the type, like fields); a case with data
is `name: type`; no `;`, no `,`. Structs and enums share the type
namespace. The keyword is `enum` (Rust/C/Swift/TypeScript) over `variant`.

## Values

```
'a' := Status.empty;                 # a case with no data: bare
'b' := Status.claimed["Tankun"];     # a case with data: the bracket builds it
struct Locker { number: int32  status: Status }
```

- **An enum value is always owned**, like a struct (one rule; adding a
  `str` case later breaks nothing): `give` / `copy` / `lend` apply, even
  to `Status.locked`.
- **`==` works**: same case and same data. `if 's' == Status.empty { }`
  when one case matters.

## `match` — when every case matters

```
fn describe [lent 'l': Locker] -> str.utf8 {
    match 'l'.status {
        empty         { return "free"; }
        claimed 'who' { return "taken by " ++ 'who'; }
        locked        { return "locked"; }
    }
}
```

- The arms are the same shape as `otherwise { … }` (`failures.md`): the
  case's bare name, then a quoted name for its data if it has any, then a
  block. **Every case must be covered** — adding a case to the enum breaks
  every `match` on purpose, as adding a failure breaks every caller.
- `match` is a **statement**, not a value (blocks are statements): arms
  `return`, or assign to a variable declared before.
- The word: `match` — the value is matched against the arms' shapes; the
  word Rust, Swift, OCaml, Python use; `switch` carries C's fall-through
  baggage, `when` reads as time (and Xag used it differently).
- `match` on a non-enum is an error ("use `if`").

## Oracle

Enforces all of the above (`examples/enums.cpls`, thirteen reject tests).
Note for the compiler: `Status.claimed["x"]` and `shapes.area[…]` have the
same shape; the oracle resolves by name at run time, the compiler by the
symbol table — no ambiguity, since a type and a module cannot share a name
in one scope (open: state that rule).

## Open

- A catch-all arm — deliberately not offered; exhaustiveness is the point.
- A `match` that yields a value.
