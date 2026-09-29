# Generics — `<T>`, a blank the compiler fills

Decided 2026-09-20. Owner asked "how's it without generics?" — you write
the same struct or function once per type, by hand, and fix every bug in
every copy (C; Go for ten years). With generics you write it once with a
blank, and the compiler fills the blank per use.

## The model (decided with variadics, 2026-09-19)

A generic is written once and **compiled per type it is used with** —
Precog sees every call site, so `max[3, 9]` and `max[2.5, 1.5]` become an
`int32` version and a `bin64` version of one source. Same speed as the
hand-written copies. This is the machinery variadics already use; Zig's
comptime and Rust's monomorphisation do the same.

## Spelling

```
fn max ['a': <T>, 'b': <T>] -> <T> { … }              # every mention wears the < >
fn first [lent 'xs': list[<T>]] -> <T> or fail[empty] { … }

struct Pair { left: <A>  right: <B> }                  # blanks in order of first appearance
enum Maybe { none  some: <T> }

'p': Pair[int32, str.utf8] = Pair[left = 7, right = "seven"];   # filled at the use
'm' := Maybe.some[42];                                          # filled from the value
```

- **`<T>` is a type variable — a blank.** Owner's spelling, over Claude's
  declared list `fn max [type T] […]`; **every mention wears the `< >`**
  (owner: "both use `<>`"), so there is no first-vs-later rule. `<` and
  `>` are operator characters, so `-> <T>` needs its space (`-><T>` is one
  operator token); a type variable's name is bare.
- **Functions**: nothing after the name — the compiler fills the blank
  from the arguments; nobody writes `max[int32][…]`. All mentions of one
  `<T>` in a signature are one blank: `max[1, "two"]` is a compile error
  ("one blank, one type").
- **Structs and enums** are used with the types spelled out —
  `Pair[int32, str.utf8]` — and their blanks are numbered **by first
  appearance in the fields** (owner: "no bracket, first appearance", over
  a declared `struct Pair [<A>, <B>]`). Reordering fields reorders the
  arguments; that is the quiet cost accepted.
- A blank cannot be sized (`size.of[<T>]`) or cross to C.

## Constraints

`<T: Ordered>` — see `interfaces.md` (decided later the same day). A
`<T>` without a bound is constrained by its body, checked at each use.

## Verbs on a `<T>`

Generic code writes the verbs for the owned case (`return give 'a'`); for
a plain `T` they do nothing (`interfaces.md`).

## A generic `op` (owner, 2026-09-29)

```
op ~= ['a': <T: Number>, 'b': <T>] -> bool { … }   # once, for any type keeping Number's promises
```

An operator may be generic, with the rules of a generic `fn`: `<T>` on
every mention, `<T: Interface>` to bound it (owner: allow). It cannot take
one of the language's own names (`+`, `==` …): its `<T>` could be
`int32`, and those are never changed for the language's own types
(design claude's consequence of 2026-09-20). **A generic one and a
specific one covering the same type are an error** (owner: a), like any
two meanings of one operator for the same types (2026-09-23): the specific
one does not quietly win, since adding it would change what existing lines
do. To give one type its own meaning, bound the generic one so that type
does not keep it. Proposed alternative: the specific one wins
(specialisation, as C++ templates do and Rust is working towards). Two
generic ones of one name are refused (the oracle's reading: they may meet
on some type). `examples/generic_op.cpls`.

## Not decided

- A generic `const`.

## Oracle

Binds each `<T>` to the first value's kind and checks every later one
(kind, not width — the compiler does the real per-type specialisation);
checks `Pair[int32, str.utf8]` against a value field by field with the
blanks substituted. `examples/generics.cpls`, eight reject tests.
