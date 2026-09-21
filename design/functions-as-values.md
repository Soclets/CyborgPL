# Functions as values

Decided 2026-09-21, three questions. What it is for (owner asked): handing
behaviour to someone else — a sort's comparison, a callback a library
runs later, the test in a `keep`/`filter`, C's `qsort`. Without it every
such thing is "copy the loop, change one line" — the problem generics
solved for types, solved here for behaviour.

## 1. A function becomes a value by its bare name

```
let 'f' := average;            # the function itself; bare name, no brackets
let 'n' := 'f'[lend 's'];      # calling a held function: quoted name, then the arguments
```

The rule stands: a *value you hold* is quoted, a function's name is bare.
A bare name alone was an error before; now it is the function. `'f'[…]`
cannot be confused with indexing, because an index always says `pos`.
Operators and `extern` functions are not values (open).

## 2. The type of a function is its signature without the names

```
fn by_number [lent 'a': Locker, lent 'b': Locker] -> bool { … }     # the definition
                fn[lent Locker,      lent Locker]  -> bool           # its type

fn sort [lent.mut 'xs': list[<T>], 'less': fn[lent <T>, lent <T>] -> bool] { … }
```

Parameter kinds (`lent`, `lent.mut`, none) are part of the type — the
caller's verb has to match — as are the return type and the failure
list (`fn[int32] -> int32 or fail[oops]`). Any function of that shape
can be passed; a different shape is a compile error naming the
difference.

## 3. Inline functions capture explicitly — `with`

```
let 'limit' := 60;
let 'passing' := keep[lend 'scores', fn ['s': int32] -> bool with [lend 'limit'] { return 's' >== 'limit'; }];
let 'evens'   := keep[lend 'scores', fn ['s': int32] -> bool { return ('s' mod 2) == 0; }];   # captures nothing
```

An anonymous function is `fn` with its parameters, return, and body,
written where it is used. It sees **nothing** of the surrounding scope
unless `with [ … ]` lists it, each with a verb: `lend 'limit'` borrows
(the function value cannot outlive `'limit'` — Precog's lifetime
inference, as for a named lend), `copy` takes an independent copy,
`give` moves the variable in (it is gone outside). Owner chose this over
named-only and over inline-without-capture; every other language captures
silently, which is where "why is this borrowed" bugs come from.

## Consequences

- A captured `lend` is visible inside as a lent variable, a `copy`/`give`
  as an owned one; a capture and a parameter cannot share a name.
- `lend 'xs'[pos i]` / `copy 'xs'[pos i]` — a verb on an element — is now
  spelled (needed to pass elements to a `lent` parameter); `give` of an
  element is refused (it would leave a hole; `list.remove`).

## Open

- (Callbacks across the C boundary: decided 2026-09-21, `ffi.md`.)
- An operator as a value (`let 'f' := <+>`).
- Returning a function that captured a `lend` — the lifetime rule.

## Oracle

`examples/functions_as_values.cpls`, ten reject tests. Checks a function
value against a `fn[…]` type by count, kinds, types, return, failures;
captures are taken at creation with their verbs; `lend.mut` of an element
is not modelled.
