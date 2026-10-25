# Functions as values

> Revised 2026-10-07: code here is in the new syntax; an old spelling in the prose is history. The old → new table is `revision-2026-10-07.md`.

Decided 2026-09-21, three questions; the fourth, operators, 2026-09-29. What it is for (owner asked): handing
behaviour to someone else — a sort's comparison, a callback a library
runs later, the test in a `keep`/`filter`, C's `qsort`. Without it every
such thing is "copy the loop, change one line" — the problem generics
solved for types, solved here for behaviour.

## 1. A function becomes a value by its bare name

```
let $f := average;            # the function itself; bare name, no brackets
let $n := $f(lend $s);      # calling a held function: its name, then the arguments in `( )`
```

The rule stands: a *value you hold* is quoted, a function's name is bare.
A bare name alone was an error before; now it is the function. `'f'[…]`
calls it. Since `pos` went (2026-09-27), a list read looks the same
(`'xs'[2]`), and what the variable holds decides which it is.
`extern` functions are not values (open); operators are, as `op plus`
(2026-09-29).

## 2. The type of a function is its signature without the names

```
fn by_number [lent $a: Locker, lent $b: Locker] -> bool { … }     # the definition
                fn[lent Locker,      lent Locker]  -> bool           # its type

fn sort <T: any> [lent.mut $xs: std::list[<T>], $less: fn[lent <T>, lent <T>] -> bool] { … }
```

Parameter kinds (`lent`, `lent.mut`, none) are part of the type — the
caller's verb has to match — as are the return type and the failure
list (`fn[i32] --> i32 or fail[oops]`). Any function of that shape
can be passed; a different shape is a compile error naming the
difference.

## 3. Inline functions capture explicitly — `with`

```
let $limit := 60;
let $passing := keep(lend $scores, fn [$s: i32] -> bool with [lend $limit] { return $s at_least $limit; });
let $evens   := keep(lend $scores, fn [$s: i32] -> bool { return ($s mod.floor 2) equal_to 0; });   # captures nothing
```

An anonymous function is `fn` with its parameters, return, and body,
written where it is used. It sees **nothing** of the surrounding scope
unless `with [ … ]` lists it, each with a verb: `lend 'limit'` borrows
(the function value cannot outlive `'limit'` — the prover's lifetime
inference, as for a named lend), `copy` takes an independent copy,
`give` moves the variable in (it is gone outside). Owner chose this over
named-only and over inline-without-capture; every other language captures
silently, which is where "why is this borrowed" bugs come from.

### The short form (owner, 2026-10-24: "(c)"; not built)

```
let $passing := keep(lend $scores, fn [$s] with [lend $limit] = $s at_least $limit);
let $evens   := keep(lend $scores, fn [$s] = ($s mod.floor 2) equal_to 0);
```

One value after `=`, no `{ return …; }`; the parameters' types and the return type come from the `fn[…]` type of
the place it is passed (`keep`'s `fn[i32] -> bool`). Design claude's readings: `=` (the sketch chosen with "(c)");
captures stay explicit with `with`; with nowhere to take its types from, it is refused, naming them; the long form
stays. A named function has no short form (owner: "no"), its body always in `{ }`.

## 4. An operator is a value as `op plus` (owner, 2026-09-29)

```
let $sum     := fold(lend $xs, 0, op plus);
let $product := fold(lend $xs, 1, op times);
let $flipped := map(lend $xs, op neg);
let $f: fn[i32, i32] -> i32 = op mod.floor;
```

`op` — the word that defines an operator — before any operator makes it
a function value, for a `fold`, a sort's comparison, or a calculator
that picks one at run time. It works for every operator, glyph or word:
`op </==`, `op ++`, `op shl`, a custom `op dot`. Before this an inline
`fn` did the same (`fn ['a': int32, 'b': int32] --> int32 { return 'a' +
'b'; }`), and still does; design claude recommended keeping only that,
and the owner chose `op`. Proposed spellings not chosen: `(+)` (Haskell,
OCaml), `op[+]`.

- **The type comes from where it goes**: a parameter's type, or a typed
  declaration. `let 'f' := op plus;` alone is refused: nothing says which `+`
  (int32, bin64, a Vec). *(design claude's reading, not ruled)*
- **`op minus` is subtraction; negation is `op neg`** (owner). `neg` exists
  only after `op`; in an expression negation is still `-'x'`. Proposed
  alternatives: where it goes decides (a one-value slot negates); a
  placeholder such as `op -_` (`_` is digit grouping already).
- **`op and.then` and `op or.else` are refused** (owner): a function's
  arguments are worked out before it runs, so they could skip nothing and
  would secretly be `op and` / `op or`. The error says to write those.
- A custom operator's operands are lent unless its definition says
  `give`, so `op dot` is a `fn[lent Vec, lent Vec] --> f64` *(the
  oracle's reading)*.

## Consequences

- A captured `lend` is visible inside as a lent variable, a `copy`/`give`
  as an owned one; a capture and a parameter cannot share a name.
- **A function carrying a `copy`/`give` capture is an owned value**
  (owner, 2026-09-24: a): `let 'g' := 'f'` is refused, as for a list, and
  `copy 'f'` copies what it carries while `give 'f'` hands it on. Dropping
  it frees what it owns. A function carrying nothing, or only `lend`s,
  copies by itself like a number. Proposed alternatives: every function
  value takes a verb; no `copy`/`give` captures at all, which would undo
  2026-09-21.
- `lend 'xs'[i]` / `copy 'xs'[i]` — a verb on an element — is now
  spelled (needed to pass elements to a `lent` parameter); `give` of an
  element is refused (it would leave a hole; `list.remove` hands the element
  back — owner, 2026-09-21).

## Open

- (Callbacks across the C boundary: decided 2026-09-21, `ffi.md`.)
- (An operator as a value: decided 2026-09-29, section 4.)
- (Returning a function that captured a `lend`: the 2026-09-21 rule, "a
  lend may leave its call and Precog proves it lives long enough", covers
  it. A function value is one more borrow leaving; `ownership-syntax.md`.)

## Oracle

`examples/functions_as_values.cy`, ten reject tests. Checks a function
value against a `fn[…]` type by count, kinds, types, return, failures;
captures are taken at creation with their verbs; `lend.mut` of an element
is not modelled.
