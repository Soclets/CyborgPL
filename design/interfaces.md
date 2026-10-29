# Interfaces — a named list of promises

> Revised 2026-10-07: code here is in the new syntax; an old spelling in the prose is history. The old → new table is `revision-2026-10-07.md`.

Decided 2026-09-20, after the owner asked "how do you say what your
generic supports?" and chose to open interfaces now rather than leave the
body as the only constraint.

## The problem

`max` works for any type that can be compared. Numbers can; a `Vec`
can't unless someone teaches it. So a generic needs a way to say "any type
that can be compared," and a type needs a way to *be* comparable.

## 1. A type may define the language's operators for itself (owner: yes)

```
op < [lent $a: Vec, lent $b: Vec] -> bool:
    …
;op <
op == [lent $a: Vec, lent $b: Vec] -> bool:
    …
;op ==
op + [lent $a: Vec, lent $b: Vec] -> Vec:
    …
;op +
```

Revises 2026-09-19's "the language's operators cannot be redefined": they
cannot be *changed* for the language's own types (`op plus ['a': i32, …]`
is an error), but a struct or enum may define them for itself, with the
operator's normal precedence. This is what lets `'u' plus 'v'` read like
arithmetic and what an interface needs. `not`, `and`, `or` stay the
language's.

## 2. The promise — `interface`

```
sig Ordered [<T>]:
    op < [lent $a: <T>, lent $b: <T>] -> bool;
    op == [lent $a: <T>, lent $b: <T>] -> bool;
;sig Ordered
sig Printable [<T>]:
    fn show [lent $x: <T>] -> str.utf8;
;sig Printable
set Number:
    i32;
    i64;
    f64;
;set Number
```

Top level beside `struct`; `pub` works; entries are `fn` and `op`
signatures about some `<T>` — the type the interface does not know yet,
announced after the name since 2026-10-02 —
ending with `;`, no bodies ("an interface lists promises, not code").
The name is the author's: `Ordered`, `Comparable`, whatever; none is
built in (whether std ships standard ones is a std question).

## 3. Keeping it — automatic, with an optional checked claim (owner: c, then `satisfies`)

A type keeps a promise **by having the operations**: `Vec` with an
`op <` and an `op ==` *is* `Ordered`; nothing to declare (Go's way). The
language's types keep the promises their built-in operators make (numbers
are `Ordered`; text is not, since `<` on text is an error).

Optionally, a **verified claim**:

```
Vec satisfies Ordered;
```

It does not make `Vec` ordered; it asks the compiler to check that it is,
and errors at the definition rather than at some later use — the
`!mut` / `check` pattern: statable, verified, not standard. The word is
`satisfies` (owner, over `is`: "Vec is Iterator" reads oddly for
noun-named interfaces; `satisfies` reads right for any name).

## 4. Asking — `<T>: Ordered`

```
fn max [<T>: Ordered] [let $a: <T>, let $b: <T>] -> <T>:
    if $b < $a:
        ret give $a;
    ;if
    ret give $b;
;fn max
```

The bound is written once, where the blank is announced after the name
(2026-10-02; until then on any one mention, `'a': <T: Ordered>`); several
join with `and`, and `any` is said outright (`generics.md`). A call
whose type does not keep the promise is a compile error at the call:
`max["a", "b"]` → "str is not `Ordered`: no `op <` for it". Inside the
body, only the promised operations may be used on a `<T>` (the compiler
checks this against the interface; the oracle does not).

## Verbs on a `<T>`

A `<T>` may be filled with an owned type or a plain one, so generic code
is written for the owned case — `return give 'a'`, `copy 'x'` — and for a
plain `T` the verb does nothing (the prover specialises it away). Without this
rule no generic body could return its parameter. Consequence set by
Claude 2026-09-20, from "verbs on a plain value are an error" meeting
generics.

## A list of types (owner, 2026-10-02: "both")

An interface may also list types instead of operations, one per line
like enum cases (owner, 2026-10-02: "Or just let it fucking go down"):
`interface Number {` then `i8`, `i16`, `i32`, … each on its own
line, then `}`. It accepts exactly those, and
a type written later can never join it; a list of operations stays open
to future types. The writer picks per interface. **Both may mix**: one
interface can list types and operations, and a type must be listed and
have the operations; and a generic can take several bounds with `and`,
`<T>: Number and Hashable` (owner, 2026-10-02: "Both works"; the bound after the bracket, owner: "`<A>: any`"). **An
interface announces its `<T>` after its name**, as every generic now is
(owner, 2026-10-02): `interface Ordered <T> { … }`. "Any type"
is the built-in `any` (`generics.md`). Not built yet.

**Since 2026-10-27 they do not mix** (owner, 2026-10-27: "your recom", after "Well, would (b) make it inaccurate?"). A
list of operations is a `sig`, a list of types a `set` (`set Number:` … `end of set Number`), and a blank needing both
joins them with `and`: `<T: Whole and Ordered>`. A `sig` with a type entry, a `set` with an operation, and a `sig` of
types only are each refused, naming the other.

**A `set` gives a body only its types** (owner, 2026-10-28: "(b)"): an operator on a `<T>` value comes from a `sig` in T's
bound that lists it, so a body comparing needs `<T: Whole and Ordered>`; `any` gives nothing. The signature then says
what the body uses.

## Open

- Std's standard interfaces.
- Interfaces with associated types (an `Iterator` whose item type varies).
- Whether an interface may promise a `const` or a field.

## Oracle

`examples/interfaces.cy`, nine reject tests. Satisfaction is checked
by shape (kind or struct/enum name), for claims at load and for bounds
when a call fills the blank; an operator defined for several types picks
the definition by its left operand's type; the body-only-uses-promised
check is not modelled.
