# Control flow

Decided 2026-09-14. Conditions are values (`[ ]`), bodies are `{ }`,
keywords carry `lang:`, and the chain narrows where there is a variant.

## Branching

```
lang:if ['x' > 10] {
    ...
} lang:else.if ['x' > 5] {
    ...
} lang:else {
    ...
}
```

- The condition must be a `bool`. No truthiness: `lang:if ['x']` on an
  `int32` is a compile error.
- `lang:else.if` narrows `else` with a condition — the same rule as
  `lend.mut`.
- Braces are mandatory, even around one statement.
- Precog narrows the range of every variable inside a branch by its
  condition; this is where guardrails live.

## Loops

`loop` is the general case, narrowed by how it ends:

```
lang:loop {                                          // forever, until lang:break
    ...
}

lang:loop.while ['count' < 100] {                    // while a condition holds
    ...
}

lang:loop.for 'item' lang:in [lang:lend 'items'] {   // each element of a list
    ...
}
```

- `lang:break;` and `lang:continue;` as usual.
- In `loop.for`, `'item'` is a fresh variable each iteration. How each
  element arrives is the same ownership question as anywhere else, using the
  same verbs: `lang:lend 'items'` lends each element; `lang:give 'items'`
  consumes the list.
- For Precog: `loop.for` over a list has a bound for free (the length);
  `loop` and `loop.while` are where invariant inference or a
  programmer-supplied bound is needed for `check` to succeed.

## Open

- `lang:in` as the separator in `loop.for`, or another spelling.
- Counting loops need a range value: a range operator (`[0 .. 10]`,
  inclusive/exclusive to decide) or a std function
  (`std::range[0, 10]`). Claude recommends starting with the std function.
