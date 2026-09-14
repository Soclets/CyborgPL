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

A block is a scope. A variable declared in it is gone when it ends. **No
shadowing** (decided 2026-09-14): a name visible from an enclosing scope
cannot be redeclared in an inner one — nothing is ever silently hidden.

## Loops

`loop` is the general case, narrowed by how it ends:

```
loop {                                          // forever, until break
    ...
}

loop.while ['count' < 100] {                    // while a condition holds
    ...
}

loop.for 'item' in [lend 'items'] {   // each element of a list
    ...
}
```

- `break;` and `continue;` as usual.
- In `loop.for`, `'item'` is a fresh variable each iteration. How each
  element arrives is the same ownership question as anywhere else, using the
  same verbs: `lend 'items'` lends each element; `give 'items'`
  consumes the list.
- For Precog: `loop.for` over a list has a bound for free (the length);
  `loop` and `loop.while` are where invariant inference or a
  programmer-supplied bound is needed for `check` to succeed.

## Open

- Counting loops need a range value: a range operator (`[0 .. 10]`,
  inclusive/exclusive to decide) or a std function
  (`std::range[0, 10]`). Claude recommends starting with the std function.
