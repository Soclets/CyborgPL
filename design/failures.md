# Failures — how a function says "this didn't work"

Decided 2026-09-19, one question at a time.

## The model: failure is a value the caller must handle

A function that can fail says so in its signature and hands the failure back
as a value. The caller **must** say what happens before the value can be
used; forgetting is a compile error. No exceptions (implied control flow —
against "nothing is implied"), no sentinel values nobody checks. Rust's
`Result` and Zig's error sets landed here for the same reasons; Precog sees
every failure path as it sees any other value.

## A failure has a name and may carry data

Each way a function can fail is a **bare name** — a reason, not a variable —
and may carry a payload of any type. The reasons are listed **inline, in the
function's own signature** (owner: option 1, over a named reusable type):

```
fn stash [lent.mut 'l': Locker, 'item': str.utf8] -> bool or fail[full, bad_item: str.utf8] {
    if free_slots[lend 'l'] == 0 { fail full; }
    if 'item' == "" { fail bad_item[copy 'item']; }
    list.append[lend.mut 'l'.items, give 'item'];
    return true;
}

fn save [lent 'l': Locker] -> fail[disk_full, no_permission] { … }   # nothing on success, or a failure
```

- `-> T or fail[…]`: "returns a T, or fails one of these ways."
  `-> fail[…]` alone: nothing on success. The list is comma-separated
  names; a name with a payload is `name: type`, the parameter shape.
- **`fail name;`** leaves the function with that failure — its own keyword,
  so a failure exit reads differently from `return`. A payload goes in
  brackets: `fail bad_item["rope"];` (the bracket builds, as everywhere).
- A function may only `fail` with a name in its list; a function whose
  signature has no `fail[…]` cannot `fail` at all; both are compile errors.
- `MAIN` cannot fail; it exits.

Because the reasons live on the function, there is no `enum` keyword and no
general variant type yet — "a locker is empty / claimed / locked" still has
no spelling outside a failure (open item 4).

## The caller: `otherwise`

A call that can fail carries **`otherwise`** and one of four things. The
call must be the whole right-hand side (or the whole statement); a failing
call buried in a larger expression is an error — declare it on its own
line first (`let 'v' := f[…] otherwise 0;`).

```
let 'ok' := stash[lend.mut 'mine', copy 'n'] otherwise false;          # a fallback value
let 'ok' := stash[lend.mut 'mine', copy 'n'] otherwise return false;   # leave, whatever the reason
let 'ok' := stash[lend.mut 'mine', copy 'n'] otherwise fail;           # pass it up unchanged
let 'ok' := stash[lend.mut 'mine', copy 'n'] otherwise {               # one arm per reason
    full           { break; }
    bad_item 'why' { std::print.stdout["bad: ", 'why', \n]; continue; }
};
save[lend 'mine'] otherwise { disk_full { std::print.stdout["not saved", \n]; } };
```

- **A value** — used instead. **A leaving statement** (`return …`, `fail …`,
  `break`, `continue`, `std::exit[…]`) — for every reason alike. **`fail`** — the failure
  climbs to *my* caller; my own `fail[…]` must list every name the callee
  can fail with. **Arms** — every reason the callee lists must be covered
  (adding a failure to a function breaks every caller on purpose); a payload
  is bound to a quoted name, which is now a variable you hold.
- In the arms form, if the call's value is used (`let 'ok' := …`) each arm
  must leave — there is no value to give `'ok'`. If the call is a statement,
  arms may fall through.
- A call that cannot fail must not have `otherwise`.
- The word is `otherwise`, not `or`: `or` is the boolean operator and
  `stash[…] or false` would be unreadable without knowing `stash`'s type
  (Claude caught the collision after the owner had said `or`; candidates
  were `else`, `catch`, `otherwise`, `onfail`; owner: `otherwise`).

## The oracle

Enforces all of it, and checks the caller's side *before* running the call,
as the compiler would: an unhandled failing call, `otherwise` on a safe call,
a missing or unknown arm, a propagation the enclosing function does not
list, a payload of the wrong type. `examples/failures.cpls`, eighteen reject
tests.

## Open

- Whether std's boundary functions (input, files) return failures this way
  (they should) and what their reason lists are.
- A catch-all arm (`other { … }`) — deliberately not offered: exhaustiveness
  is the point.
