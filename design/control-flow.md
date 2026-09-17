# Control flow

Decided 2026-09-14. Conditions are values (`[ ]`), bodies are `{ }`, and
the chain narrows where there is a variant.

## Branching

```
if ['x' > 10] {
    ...
} else.if ['x' > 5] {
    ...
} else {
    ...
}
```

- The condition must be a `bool`. No truthiness: `if ['x']` on an
  `int32` is a compile error.
- `else.if` narrows `else` with a condition — the same rule as
  `lend.mut`.
- Braces are mandatory, even around one statement.
- Precog narrows the range of every variable inside a branch by its
  condition; this is where guardrails live.

## Scopes

A block is a scope. A variable declared in it is gone when it ends.
**Shadowing is never silent** (decided 2026-09-14): a name visible from an
enclosing scope cannot be redeclared with `:=` or `=`. It *can* be
deliberately hidden with `$:=` / `$=`, the `$` marking the intent:

```
var 'x' := [1];
if [true] {
    var 'x' $:= [2];          // shadows the outer 'x' until the block ends
    var.int32 'y' $= [3];     // error: there is no 'y' to shadow
}
```

An `immut` variable cannot be shadowed.

## Loops

`loop` is the general case, narrowed by how it ends — and, for `for` and
`range`, by whether the loop's variable outlives it (decided 2026-09-17):

```
loop {                                              // forever, until break
    …
}

loop.while ['count' < 100] {                        // while a condition holds
    …
}

loop.temp.for 'item' in [lend 'items'] {            // each element; 'item' gone after
    …
}

loop.temp.range 'i' [:= 0, := 1000] {               // 'i' from 0 to 1000 inclusive
    …
}
```

### `temp` / `perm` — mandatory

`temp`: the variable exists only inside the loop. `perm`: it is still there
afterwards, holding **whatever the loop last gave it** — the stop value after
a full run, the value at a `break`, the start value if the body never ran.
One rule for every way a loop can end.

```
loop.perm.range 'i' [:= 0, := 100] {
    if [found['i']] { break; }
}
std::printu.stdout["stopped at " 'i' \n];
```

`perm` on `for` keeps the last element visited (an empty list is an error:
there is nothing to keep).

### `range`

`[:= start, := stop]` — **inclusive**: "start from, stop at"; the writer sees
`'i'` reach `stop`. A third slot is the **step**: `[:= 0, := 100, := 10]`;
negative counts down: `[:= 10, := 0, := -1]`; a step of 0 is a compile error.
Start past stop with a positive step runs zero times.

The typed form puts the type last in the chain and `=` on the bounds, like
`var`: `loop.temp.range.int32 'i' [= 0, = 1000]`. Every value of `'i'` must
fit the type.

**The body cannot assign to the counter** (or to a `for` element): it belongs
to the loop, so the header alone says how many times the body runs and what
`'i'` is on every pass — which is what keeps `range` loops easy for Precog.
Skipping ahead is a `loop.while` with your own counter.

### Chain order

`loop . temp|perm . [seq .] for|range [. type]` — `seq` (opt out of automatic
parallelism) sits before the kind; the type, when given, is last:
`loop.temp.seq.range.int32 'i' [= 0, = 9]`.

### Not in the language

Labelled `break` (leave an outer loop from an inner one: use a flag, or a
`perm` counter) and character ranges — both considered and skipped,
2026-09-17. Index-alongside-element in `for` is parked until lists have
indexing.

- `break;` and `continue;` as usual.
- How each element of a `for` arrives is the ownership question with the
  same verbs: `lend 'items'` lends each element; `give 'items'` consumes the
  list.
- For Precog: `range` and `for` have a bound for free; `loop` and
  `loop.while` are where invariant inference or a programmer-supplied bound
  is needed for `check` to succeed.

## Open

