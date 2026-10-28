# Enums — one of a few named cases

> Revised 2026-10-07: code here is in the new syntax; an old spelling in the prose is history. The old → new table is `revision-2026-10-07.md`.

Decided 2026-09-20. Owner asked "what are enums?" — a type that means *one
of these few options*, like `bool` with your own names, where an option may
carry a detail. A function's `fail[…]` list has this shape already
(`failures.md`); `enum` is the same idea as a type you can hold.

## Declaring

```
union.tag Status:
    empty;
    claimed: str.utf8;       # this case carries who claimed it
    locked;
;union.tag Status
```

Top level beside `struct`; `pub` works the same. The name is bare (a type);
case names are bare (paths into the type, like fields); a case with data
is `name: type`; cases are separated by `,`, a `,` after the last allowed
(owner, 2026-10-14; until then there was none). Structs and enums share the type
namespace. The keyword is `enum` (Rust/C/Swift/TypeScript) over `variant`.

## Values

```
let $a := Status.empty;                 # a case with no data: bare
let $b := Status.claimed["Tankun"];     # a case with data: the bracket builds it
rec Locker:
    var number: i32;
    var status: Status;
;rec Locker
```

- **An enum value is always owned**, like a struct (one rule; adding a
  `str` case later breaks nothing): `give` / `copy` / `lend` apply, even
  to `Status.locked`.
- **`==` works**: same case and same data. `if 's' == Status.empty { }`
  when one case matters.

## `match` — when every case matters

```
fn describe [lent $l: Locker] -> str.utf8:
    match $l.status:
        case empty:
            ret "free";
        ;case empty
        case claimed $who:
            ret "taken by " concat $who;
        ;case claimed
        case locked:
            ret "locked";
        ;case locked
    ;match
;fn describe
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

Enforces all of the above (`examples/enums.cy`, thirteen reject tests).
Note for the compiler: `Status.claimed["x"]` and `shapes.area[…]` have the
same shape; the oracle resolves by name at run time, the compiler by the
symbol table — no ambiguity, since a type and a module cannot share a name
in one scope (open: state that rule).

## A `match` that gives a value (owner, 2026-09-29)

```
let $label := match $s:
    case empty:
        "free"
    ;case empty
    case claimed $who:
        "taken by " concat $who
    ;case claimed
    case locked:
        ret "(locked)";
    ;case locked      # a case may still leave instead
;match;
```

A case whose block holds only a value is that value (a). A block is
either one value or statements that leave, never a mix, so there is no
question of which line was the value. Every case is still covered, and a
case that gives a value in a `match` used as a statement is an error.
The same holds for `otherwise` with several reasons:
`to.i32['t'] otherwise { not_a_number { -2 } out_of_range { -1 } }`.

**In a `#strict` file a case says it with `giveS`** (b; spelled `gives`
until the owner renamed it the same day, the capital S keeping it apart
from `give`): `empty { giveS "free"; }`, and it may do other work first.
`giveS` belongs to strict
files, as `pos` does, and the bare value is refused there. Proposed
alternative: an arrow, `empty -> "free"`, which collides with `->` for
return types. "`match` is a statement" (2026-09-20) was design claude's
detail, not a ruling. The owner asked what `match` is for and why the
word, since that is `enums.md` above. `examples/match_value.cy`,
`examples/strict_match.cy`.

**A `match` alone in a case: the outer match decides** (owner,
2026-09-29: a). In a `match` used as a statement, a case holding only
another `match` runs it as a statement; in a `match` used as a value, it
is that match's value. In a `#strict` file a bare nested `match` is
always a statement, since a value says `giveS` (owner: "yes"):

```
match $o:
    case cancelled:
    ;case cancelled
    case paid:
        match $s:
            case pickup:
                …
            ;case pickup
            case post:
                …
            ;case post
        ;match
    ;case paid                 # runs
;match
let $price := match $o:
    case cancelled:
        0
    ;case cancelled
    case paid:
        match $s:
            case pickup:
                5
            ;case pickup
            case post:
                9
            ;case post
        ;match
    ;case paid
;match;
#strict: paid { giveS match 's' { pickup { giveS 5; } post { giveS 9; } }; }
```

Raised by check claude (the engines split). Proposed alternatives: a
`match` where a statement can go is always a statement (b); a statement
`match` alone in a case ends with `;` (c, Rust's escape hatch). Rust
treats the inner match as the case's value and errors unless its value
is `()`. Alternative words for `giveS`: `yield` (Java's switch), `is`
(recommended by design claude), `value`, `result`.

## Open

- A catch-all arm — deliberately not offered; exhaustiveness is the point.
