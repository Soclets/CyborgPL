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

let 'p': Pair[int32, str.utf8] = Pair[left = 7, right = "seven"];   # filled at the use
let 'm' := Maybe.some[42];                                          # filled from the value
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

## Not decided

- **Constraints** — saying a `<T>` must support `>` or `++` (Rust's
  trait bounds, Swift's protocols). Today a generic body is checked at
  each use, like a variadic's: `max["a", "b"]` fails to compile at that
  call because `>` on text is an error. Precog-style "the body is the
  constraint" (Zig) may be enough; a spelled constraint is open.
- A generic `op`; a generic `const`.

## Oracle

Binds each `<T>` to the first value's kind and checks every later one
(kind, not width — the compiler does the real per-type specialisation);
checks `Pair[int32, str.utf8]` against a value field by field with the
blanks substituted. `examples/generics.cpls`, eight reject tests.
