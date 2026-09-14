# Ownership syntax

Decided 2026-09-14. Semantics are in `core-features.md` (one owner per
value; moves, copies, borrows; Rust-style borrow checking; inferred
lifetimes with optional verified annotation).

## Explicit, from the owner's perspective

Handing an **owned** value (heap types: `str`, lists, `char.grapheme`,
`std::uer`, …) to somewhere else says what happens to it. A bare
`'s' = ['t'];` on an owned value is a compile error.

```
's' = [give 't'];        // ownership transfers; 't' is dead after this
's' = [copy 't'];        // a second, independent value; an allocation
process[lend 's'];       // temporary shared access; 's' still owns
process[lend.mut 's'];   // temporary exclusive access, may modify
```

- **`give`** — move. I give it away.
- **`copy`** — duplicate.
- **`lend`** — borrow, shared. **`lend.mut`** — borrow, mutable; the chain
  narrows `lend` the usual way.

The words are the *owner's* — the keyword sits on the value being handed
over, so "give" and "lend" read as what the owner does. (Rust's `move` /
`borrow` are the receiver's words; the concepts are identical.)

**Plain values** — `int32`, `bin64`, `bool`, `char.point`, … — own nothing,
so `'s' = ['t'];` simply copies with no keyword. There is no other sensible
meaning for a number.

The same verbs apply at call sites, which settles most of the borrow-syntax
question: a parameter receives whatever the caller wrote.

## Why explicit

Rust moves silently; the line that killed a variable says nothing, and the
error appears two lines later. In CyborgPL nothing is implied, and these are
exactly the assignments that cost something at runtime — so they are the
ones that should be visible.

## The parameter side (decided 2026-09-14)

A parameter declares what it accepts, so the signature *is* the ownership
contract:

```
func.int32 count [lent.str.utf8 'text'] { ... }       // looks at it
func edit        [lent.mut.str.utf8 'text'] { ... }   // may modify it
func keep        [var.str.utf8 'text'] { ... }        // owns it
```

- **`lent`** — receives a shared borrow. The caller writes `lend`.
- **`lent.mut`** — receives a mutable borrow. The caller writes
  `lend.mut`.
- **`var`** — receives ownership. The caller writes `give` or
  `copy`.

Caller verb and parameter kind must agree; a mismatch is a compile error
with an obvious message ("`count` only looks at `'text'`; you tried to give
it away"). A reader knows from the signature alone whether their value comes
back, and Precog has the contract without the body.

The words pair: the caller *lends*, the parameter is *lent*.

A value with no owner yet — a literal or a call's result — may go straight
into a `lent` parameter: `count[["a", "b"]]`. It lives for the call and is
freed after; nothing else can reach it, so nothing is implied. (Decided
2026-09-14.)

## Open

- The lifetime annotation's spelling (optional, verified).
- Whether `copy` on a plain value is allowed as a no-op (for
  uniformity) or rejected as noise.
