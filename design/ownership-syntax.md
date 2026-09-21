# Ownership syntax

Decided 2026-09-14. Semantics are in `core-features.md` (one owner per
value; moves, copies, borrows; Rust-style borrow checking; inferred
lifetimes with optional verified annotation).

## Explicit, from the owner's perspective

Handing an **owned** value (heap types: `str`, lists, `char.grapheme`,
`std::uenr`, …) to somewhere else says what happens to it. A bare
`'s' = 't';` on an owned value is a compile error.

```
's' = give 't';          # ownership transfers; 't' is dead after this
's' = copy 't';          # a second, independent value; an allocation
process[lend 's'];       # temporary shared access; 's' still owns
process[lend.mut 's'];   # temporary exclusive access, may modify
```

- **`give`** — move. I give it away.
- **`copy`** — duplicate.
- **`lend`** — borrow, shared. **`lend.mut`** — borrow, mutable; the chain
  narrows `lend` the usual way.

**Alternative spelling (owner, 2026-09-21): `&` for `lend` / `lent`, `&mut`
for `lend.mut` / `lent.mut`**, on both sides of a call —
`count[&'s']`, `process[&mut 's']`, `fn count [&'text': str.utf8]`,
`fn edit [&mut 'text': str.utf8]`, `fn[&str.utf8] -> int32`. Same
meaning, same rules; a program may mix the two. Added after four cold
readers in a row confused `lent` with `lend` — `&` is the spelling Rust
readers already know. `&` between two values is still an (undefined)
custom operator; the lending `&` is the one that opens a value. A build
that sees `&` or `&mut` anywhere prints, once:
`What are you? Rust programmer or Claude?` — and builds anyway (owner).

The words are the *owner's* — the keyword sits on the value being handed
over, so "give" and "lend" read as what the owner does. (Rust's `move` /
`borrow` are the receiver's words; the concepts are identical.)

**Plain values** — `int32`, `bin64`, `bool`, `char.point`, … — own nothing:
the value sits directly in the variable, with no allocation behind it to be
responsible for. So `'s' = 't';` simply copies with no keyword, and
`give`/`copy`/`lend` on a plain value is a **compile error** (decided
2026-09-17): the verb would promise a transfer of a responsibility that does
not exist.

The same verbs apply at call sites, which settles most of the borrow-syntax
question: a parameter receives whatever the caller wrote.

## Mutability: statable, not standard (decided 2026-09-14)

A plain `let 'x' := 1000;` is the standard practice — Precog already knows
whether `'x'` is ever reassigned, so nothing needs saying. The programmer
*may* state it, as a bare word before the name (revised 2026-09-19; it was
a chain segment, `var.immut.int32`), and the compiler then holds them to it:

```
let immut 'limit': int32 = 10;      # assigning to 'limit' later is an error
let mut 'counter' := 0;             # states the intent; nothing to enforce
fn twice [immut 'n': int32] -> int32 { … }
```

`immut` is a verified contract, like `check` and the optional lifetime
annotation. `mut` is documentation. Neither is expected in ordinary code.

Not the same thing as a constant (a value known at compile time, baked into
the binary): Precog finds those itself; no keyword exists for them.

## Why explicit

Rust moves silently; the line that killed a variable says nothing, and the
error appears two lines later. In CyborgPL nothing is implied, and these are
exactly the assignments that cost something at runtime — so they are the
ones that should be visible.

## The parameter side (decided 2026-09-14)

A parameter declares what it accepts, so the signature *is* the ownership
contract:

```
fn count [lent 'text': str.utf8] -> int32 { … }    # looks at it
fn edit  [lent.mut 'text': str.utf8] { … }         # may modify it
fn keep  ['text': str.utf8] { … }                  # owns it
```

- **`lent`** — receives a shared borrow. The caller writes `lend`.
- **`lent.mut`** — receives a mutable borrow. The caller writes
  `lend.mut`.
- **No keyword** — receives ownership. The caller writes `give` or
  `copy`. (Until 2026-09-19 this was `var.str.utf8 'text'`; with the type
  after the name the owning case needs no word.)

The verb sits before the name on both sides, so the definition mirrors the
call: `count[lend 's']` ↔ `[lent 'text': …]`. Rust puts the borrow in the
type (`text: &str`) and mirrors it at the call with `&s`; the owner chose the
mirror over the type-position, 2026-09-19.

Caller verb and parameter kind must agree; a mismatch is a compile error
with an obvious message ("`count` only looks at `'text'`; you tried to give
it away"). A reader knows from the signature alone whether their value comes
back, and Precog has the contract without the body.

The words pair: the caller *lends*, the parameter is *lent*.

A value with no owner yet — a literal or a call's result — may go straight
into a `lent` parameter: `count[["a", "b"]]`. It lives for the call and is
freed after; nothing else can reach it, so nothing is implied. (Decided
2026-09-14.)

A lend may be given its own name (decided 2026-09-14):
`let 'r' := lend 's';` — `'r'` looks at `'s'` without owning it: readable,
not assignable, not give-able. Precog's lifetime inference keeps it from
outliving `'s'`.

## Open

- The lifetime annotation's spelling (optional, verified). (Reaffirmed 2026-09-21:
  a lend *may* escape a call and Precog proves it; the rule "a lend never
  outlives its scope" was offered and declined.)
