# Interfaces — a named list of promises

Decided 2026-09-20, after the owner asked "how do you say what your
generic supports?" and chose to open interfaces now rather than leave the
body as the only constraint.

## The problem

`max` works for any type that can be compared. Numbers can; a `Vec`
can't unless someone teaches it. So a generic needs a way to say "any type
that can be compared," and a type needs a way to *be* comparable.

## 1. A type may define the language's operators for itself (owner: yes)

```
op < ['a': Vec, 'b': Vec] -> bool { … }
op == ['a': Vec, 'b': Vec] -> bool { … }
op + ['a': Vec, 'b': Vec] -> Vec { … }
```

Revises 2026-09-19's "the language's operators cannot be redefined": they
cannot be *changed* for the language's own types (`op + ['a': int32, …]`
is an error), but a struct or enum may define them for itself, with the
operator's normal precedence. This is what lets `'u' + 'v'` read like
arithmetic and what an interface needs. `not`, `and`, `or` stay the
language's.

## 2. The promise — `interface`

```
interface Ordered {
    op < ['a': <T>, 'b': <T>] -> bool;
    op == ['a': <T>, 'b': <T>] -> bool;
}
interface Printable {
    fn show [lent 'x': <T>] -> str.utf8;
}
```

Top level beside `struct`; `pub` works; entries are `fn` and `op`
signatures about some `<T>` — the type the interface does not know yet —
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
and errors at the definition rather than at some later use — the `let
immut` / `check` pattern: statable, verified, not standard. The word is
`satisfies` (owner, over `is`: "Vec is Iterator" reads oddly for
noun-named interfaces; `satisfies` reads right for any name).

## 4. Asking — `<T: Ordered>`

```
fn max ['a': <T: Ordered>, 'b': <T>] -> <T> {
    if 'b' < 'a' { return give 'a'; }
    return give 'b';
}
```

The bound is written once, on any mention of the blank (repeating it is
allowed; a different interface on the same blank is an error). A call
whose type does not keep the promise is a compile error at the call:
`max["a", "b"]` → "str is not `Ordered`: no `op <` for it". Inside the
body, only the promised operations may be used on a `<T>` (the compiler
checks this against the interface; the oracle does not).

## Verbs on a `<T>`

A `<T>` may be filled with an owned type or a plain one, so generic code
is written for the owned case — `return give 'a'`, `copy 'x'` — and for a
plain `T` the verb does nothing (Precog specialises it away). Without this
rule no generic body could return its parameter. Consequence set by
Claude 2026-09-20, from "verbs on a plain value are an error" meeting
generics.

## Open

- Std's standard interfaces.
- Interfaces with associated types (an `Iterator` whose item type varies).
- Whether an interface may promise a `const` or a field.

## Oracle

`examples/interfaces.cpls`, nine reject tests. Satisfaction is checked
by shape (kind or struct/enum name), for claims at load and for bounds
when a call fills the blank; an operator defined for several types picks
the definition by its left operand's type; the body-only-uses-promised
check is not modelled.
