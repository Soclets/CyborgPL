# Generics — `<T>`, a blank the compiler fills

> Revised 2026-10-07: code here is in the new syntax; an old spelling in the prose is history. The old → new table is `revision-2026-10-07.md`.

Decided 2026-09-20. Owner asked "how's it without generics?" — you write
the same struct or function once per type, by hand, and fix every bug in
every copy (C; Go for ten years). With generics you write it once with a
blank, and the compiler fills the blank per use.

## The model (decided with variadics, 2026-09-19)

A generic is written once and **compiled per type it is used with** —
the prover sees every call site, so `max[3, 9]` and `max[2.5, 1.5]` become an
`i32` version and a `f64` version of one source. Same speed as the
hand-written copies. This is the machinery variadics already use; Zig's
comptime and Rust's monomorphisation do the same.

## Spelling (revised 2026-10-02)

```
function max <T: Number> [$a: <T>, $b: <T>] -> <T>:
    …
end of function max           # announced after the name, with what it accepts
function first <T: any> [lent $xs: std::list[<T>]] -> <T> or fail[empty]:
    …
end of function first
function pick <T: any> [@x] [lent @x $xs: std::list[<T>]] -> lent @x <T>:
    …
end of function pick   # then the lifetime groups, after a `|`

record Pair <A: any, B: any>:
    left: <A>;
    right: <B>;
end of record Pair         # blanks in the order announced
union.tagged Maybe <T: any>:
    none;
    some: <T>;
end of union.tagged Maybe

let $p: Pair[integer.signed.32_bit, string.utf8] = Pair[left = 7, right = "seven"];   # filled at the use
let $m := Maybe.some[42];                                          # filled from the value
```

- **`<T>` is a type variable — a blank**, and every mention wears its
  `< >` (owner, 2026-09-20: "both use `<>`"). Its name is the writer's
  (`<T>`, `<Thing>`); the brackets say it is generic, the name says which.
  `<` and `>` are operator characters, so `--> <T>` needs its space
  (`--><T>` is one operator token).
- **Every blank is announced right after the name, before its first use**
  (owner, 2026-10-02: c, then a), for functions, operators, structs, enums
  and interfaces, so a mistyped `<Y>` is an error ("`<Y>` was never
  announced") rather than a quiet second blank. One bracket per blank
  (owner: b): `<A>: any <B>: any`. It replaces 2026-09-20's "introduced by
  first use" and, for structs and enums, "numbered by first appearance in
  the fields" (owner: "no bracket, first appearance"): the order is the
  announcement's, so reordering fields no longer reorders the arguments.
- **Every blank says what it accepts**, after the bracket, as a type
  follows a variable's name (owner: "`<A>: any`"): an interface
  (`<T>: Ordered`), several joined with `and` (`<T>: Number and
  Hashable`), or the built-in `any` for a blank that needs nothing from
  its type (`Pair`, the first item of any list). Nothing is allowed by
  default (owner: "Don't default to allow all"): `<T>` with no bound is
  refused, and so is a bound on a mention instead of the announcement.
- **Lifetime groups are announced too**, after the generics, with a `|`
  between the two kinds and none within a kind (owner: "Not between every
  announcement, because generics are still the same as generics, so as
  lifetimes"): `fn pick <T>: any | @x […]`, `fn two @x @y […]`. A group
  used and never announced is refused. `< >` holds generics only.
- **Functions** are never filled by hand: the compiler fills the blank
  from the arguments (`max[3, 9]`). All mentions of one `<T>` are one
  blank: `max[1, "two"]` is a compile error ("one blank, one type").
- **Structs and enums** are used with the types spelled out —
  `Pair[i32, str.utf8]` — in the order announced, and each type must
  keep its blank's bound.
- A blank cannot be sized (`size.of[<T>]`) or cross to C.

History: from 2026-09-20 to 2026-10-02 a blank was introduced by its first
mention, a bound sat on one mention (`'a': <T: Ordered>`), a struct's
blanks were numbered by first appearance in its fields, and a lifetime
group by its first use.

## Constraints

What a blank accepts is an interface (`interfaces.md`): a list of
operations, a list of types, or both; or `any`.

## Verbs on a `<T>`

Generic code writes the verbs for the owned case (`return give 'a'`); for
a plain `T` they do nothing (`interfaces.md`).

## A generic `op` (owner, 2026-09-29)

```
operator ~= <T>: Number [$a: <T>, $b: <T>] -> boolean:
    …
end of operator ~=   # once, for any type keeping Number's promises
```

An operator may be generic, with the rules of a generic `fn`: announced
after its name with what it accepts (`op ~= <T>: Number [`, since
2026-10-02; `<T: Interface>` on a mention before) (owner: allow). It cannot take
one of the language's own names (`+`, `==` …): its `<T>` could be
`i32`, and those are never changed for the language's own types
(design claude's consequence of 2026-09-20). **A generic one and a
specific one covering the same type are an error** (owner: a), like any
two meanings of one operator for the same types (2026-09-23): the specific
one does not quietly win, since adding it would change what existing lines
do. To give one type its own meaning, bound the generic one so that type
does not keep it. Proposed alternative: the specific one wins
(specialisation, as C++ templates do and Rust is working towards). Two
generic ones of one name are refused (the oracle's reading: they may meet
on some type). `examples/generic_op.cy`.

## Not decided

- A generic `const`.

## Oracle

Binds each `<T>` to the first value's kind and checks every later one
(kind, not width — the compiler does the real per-type specialisation);
checks `Pair[i32, str.utf8]` against a value field by field with the
blanks substituted, and each filled type against its blank's bound. A
listed type is matched by family (`i32` admits any integer): widths are
the compiler's. `examples/generics.cy`, `examples/interface_types.cy`,
and the reject tests `generic_*`, `iface_*`, `lifetime_unannounced`,
`struct_bound_unkept`.
