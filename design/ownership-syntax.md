# Ownership syntax

> Revised 2026-10-07: code here is in the new syntax; an old spelling in the prose is history. The old → new table is `revision-2026-10-07.md`.

Decided 2026-09-14. Semantics are in `core-features.md` (one owner per
value; moves, copies, borrows; Rust-style borrow checking; inferred
lifetimes with optional verified annotation).

## Explicit, from the owner's perspective

Handing an **owned** value (heap types: `str`, lists, `char.grapheme`,
`std::uenr`, …) to somewhere else says what happens to it. A bare
`change 's' = 't';` on an owned value is a compile error.

```
change $s to give $t;          # ownership transfers; 't' is dead after this
change $s to copy $t;          # a second, independent value; an allocation
process(lend $s);       # temporary shared access; 's' still owns
process(lend.mut $s);   # temporary exclusive access, may modify
```

- **`give`** — move. I give it away.
- **`copy`** — duplicate.
- **`lend`** — borrow, shared. **`lend.mut`** — borrow, mutable; the chain
  narrows `lend` the usual way.
- **While something is lent `lend.mut`, nothing else may borrow it**
  (owner, 2026-10-02: a, compiler claude's question):
  `two[lend.mut 'u', lend 'u']` is refused, and so is
  `two[grow[lend.mut 'u'], lend 'u']`, where `grow` hands back a borrow of
  `'u'`. To read and change one list, pass it once with `lend.mut` (it can
  read too); to keep how it was, `copy` it first. Proposed: (a) refuse, as
  Rust does; (b) refuse only when a change actually happens while the other
  borrow is in use, as the oracle and kayo did. Compiler claude and design
  claude recommended a: the mistake shows at the call, a later change
  inside the function cannot break callers far away, and the compiler can
  tell LLVM the two never overlap.

**Old spellings since 2026-10-16**: a symbol spells only an operator that reads correctly (`operators.md`),
so `&'x'`, `&mut 'x'`, `&@x` and `&mut@x` are refused, naming `lend` / `lent` and `lend.mut` / `lent.mut`.
**Back 2026-10-14** (owner: "Yeah, what about `&`? Maybe for `and` &
`lend`/`lent`?", then "`&mut`"): `&'x'` and `&mut 'x'` spell `lend` / `lent`
and `lend.mut` / `lent.mut` again, everywhere they go, `&@x` and `&mut@x`
included; between two values `&` is `and`. Removed 2026-10-07 with MTM, they
were refused in between. Their history: **Alternative spelling (owner, 2026-09-21): `&` for
`lend` / `lent`, `&mut` for `lend.mut` / `lent.mut`**, on both sides of a call —
`count[&'s']`, `process[&mut 's']`, `fn count [&'text': str.utf8]`,
`fn edit [&mut 'text': str.utf8]`, `fn[&str.utf8] --> i32`. Same
meaning, same rules; a program may mix the two. Since 2026-10-02 it covers every
place `lent` goes (owner: "Yes."): a handed-back borrow, `-> &str.utf8` /
`--> &mut T`, and a lifetime group, `&@x 'a': T`, `&mut@x 'a': T`,
`--> &@x T`. Added after four cold
readers in a row confused `lent` with `lend` — `&` is the spelling Rust
readers already know. `&` between two values is still an (undefined)
custom operator; the lending `&` is the one that opens a value. From
2026-09-21 every `&` or `&mut` made the build print `What are you? Rust
programmer or Claude?` on stdout, one line per `&` (owner: "You're not
supposed to use that"); since 2026-10-02 it prints nothing (owner: "`&`
and `&mut` shouldn't punish anymore").

**A lend is an address** (noted 2026-09-22, from the compiler session):
this file says what a lend *means* — temporary access, the owner keeps
it — and a `lent` parameter crosses as a pointer, which is what that
is made of. Not a decision; the only shape it could have.

The words are the *owner's* — the keyword sits on the value being handed
over, so "give" and "lend" read as what the owner does. (Rust's `move` /
`borrow` are the receiver's words; the concepts are identical.)

**Plain values** — `i32`, `f64`, `bool`, `char.point`, … — own nothing:
the value sits directly in the variable, with no allocation behind it to be
responsible for. So does a `union.tag` whose every case holds no data (`Age`:
`adult`, `child`; owner, 2026-10-28: "sure"): it is plain, `fare(lend $t,
$r.age)`, and turns owned the day a case gains data, the compiler then naming
every use that needs a verb. So `change 's' = 't';` simply copies with no keyword, and
`give`/`copy`/`lend` on a plain value is a **compile error** (decided
2026-09-17): the verb would promise a transfer of a responsibility that does
not exist.

The same verbs apply at call sites, which settles most of the borrow-syntax
question: a parameter receives whatever the caller wrote.

## A lend's sizes must match (owner, 2026-09-24: a)

```
fn total [lent $xs: std::ls[i32]] -> i64:
    …
;fn total
let $big: std::ls[i64] := [1, 2, 3];
total(lend $big);          # error: a list[i64] lent where a list[i32] is looked at
```

A lend hands over the value where it is, with no copy, so what is lent
must have the parameter's layout. Range bounds do not change the layout
(a `list[i32][min: 0, max: 10]` lends as a `list[i32]`), and a `:=` collection
takes its width from its use. To take any width, make the parameter
generic. Proposed alternative: a quiet converted copy, each element a
narrowing site, written back for `lend.mut`, which would hide a copy
behind the word that means none.

## Mutability: statable, not standard (decided 2026-09-14)

A plain `let 'x' := 1000;` is the standard practice — the prover already knows
whether `'x'` is ever reassigned, so nothing needs saying. The programmer
*may* state it, as a bare word before the name (revised 2026-09-19; it was
a chain segment, `var.!mut.i32`), and the compiler then holds them to it:

```
let $limit: i32 := 10;      # assigning to 'limit' later is an error
var $counter = 0;             # a promise that it changes: refused if nothing does (2026-10-10)
fn twice [$n: i32] -> i32:
    …
;fn twice
```

`!mut` is a verified contract, like `check` and the optional lifetime
annotation. **`mut` is one too since 2026-10-10** (owner: "`b`"): a `mut`
variable or parameter that nothing in its function changes is refused, so
the word can never lie; it was documentation until then. Neither is
expected in ordinary code.

Not the same thing as a constant (a value known at compile time, baked into
the binary): the prover finds those itself, and since 2026-10-10 a body may
also say so with `const` (`kits.md`, "`pub` and `const`").

## Why explicit

Rust moves silently; the line that killed a variable says nothing, and the
error appears two lines later. In CyborgPL nothing is implied, and these are
exactly the assignments that cost something at runtime — so they are the
ones that should be visible.

## The parameter side (decided 2026-09-14)

A parameter declares what it accepts, so the signature *is* the ownership
contract:

```
fn count [lent $text: str.utf8] -> i32:
    …
;fn count    # looks at it
proc edit  [lent.mut $text: str.utf8]:
    …
;proc edit         # may modify it
fn keep  [$text: str.utf8]:
    …
;fn keep                  # owns it
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
back, and the prover has the contract without the body.

The words pair: the caller *lends*, the parameter is *lent*.

A value with no owner yet — a literal or a call's result — may go straight
into a `lent` parameter: `count[["a", "b"]]`. It lives for the call and is
freed after; nothing else can reach it, so nothing is implied. (Decided
2026-09-14.)

A lend may be given its own name (decided 2026-09-14):
`let 'r' := lend 's';` — `'r'` looks at `'s'` without owning it: readable,
not assignable, not give-able. the prover's lifetime inference keeps it from
outliving `'s'`.

## Handing back a borrow: `--> lent@x T` (owner, 2026-09-29)

```
fn longer [@x] [lent @x $a: str.utf8, lent @x $b: str.utf8] -> lent @x str.utf8:
    if std::str.bytes(lend $a) >= std::str.bytes(lend $b):
        ret lend $a;
    ;if
    ret lend $b;
;fn longer
fn pick [@x, @y] [lent @x $a: str.utf8, lent @y $b: str.utf8] -> lent @x str.utf8:
    ret lend $a;
;fn pick
```

A function may hand back a borrow of what it was lent (2026-09-21: a lend
may leave the call, and the prover proves it lives long enough). The return
says so, `--> lent T` or `--> lent.mut T`. **`@x` names a lifetime group**:
everything marked `@x` lives and dies together, so `longer`'s result is
good while both `'a` and `'b` are, and `pick`'s needs only `'a`. The name
means nothing but "same group". It is announced after the function's
name, after its generics and a `|` when there are any (owner,
2026-10-02): `fn longer @x […]`, `fn pick <T>: any | @x […]`; until then
it was introduced by first use. Rust writes the same idea `<'x>` …
`&'x str`. `'` means a variable here, so `@`, which reads as "at", and it
sits on the borrow word, since borrowing is a word before the name. A
returned borrow must come from a `lent` parameter in its group, never
from something that dies at the return, and a function returning one
must say so. Proposed alternatives: naming the parameters instead of a
group (`--> lent str.utf8 from 'a', 'b'`, recommended by design claude
and not chosen); `~` for the mark (it already means wrapping); a Rust-style
list up front (`fn longer <@x> …`). `examples/lifetimes.cy`.

**The group is optional outside `#strict`** (owner, 2026-09-29:
"`-> lent str.utf8` is legal in normal"): `fn first [lent 'a': str.utf8]
--> lent str.utf8` is fine, and the prover works out from the body what the
result depends on, as "optional, verified" (2026-09-14) had it. **A
`#strict` file must write it** (owner: "yeah, required in strict"): a
strict caller sees only signatures, and the reader should see what the
result depends on. Either way it costs nothing when the program runs.
