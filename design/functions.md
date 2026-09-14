# Functions

Decided 2026-09-14.

## Definition

```
func.int32 'add' [var.int32 'a', var.int32 'b'] {
    return ['a' + 'b'];
}
```

- The chain narrows `func` by its **return type**, the same way `var.int32`
  narrows `var`. `func 'name'` with no type: Precog picks the best return type
  from every `return` in the body. No `return` at all: the function returns
  nothing.
- **Parameters are `var` declarations** the caller fills in, comma-separated
  in one `[ ]`. `var 'a'` with no type is legal: Precog infers the parameter's
  best type from every call site in the program.
- Body in `{ }`.

## Call

```
var 'sum' := 'add'[10, 20];
```

Arguments are comma-separated values in one `[ ]`, positionally matched to
the parameters. A call is a value and may appear inside any bracket:
`var 'x' := ['add'[1, 2] * 3];`.

## Return

```
return ['a' + 'b'];
```

`[ ]` mandatory, as everywhere. Bare `return;` in a nothing-function.

## `[ ]` — the two separators

- **Whitespace** joins pieces into *one* value: `["sum: " 'sum' \n]`.
- **Comma** separates *values*: `[10, 20]`.

A bracket holding several comma-separated values is a **list**:
`var 'l' := [1, 2, 3];`. Function arguments are exactly that.

## Numbers

Digit grouping uses `_`, never `,`: `[1_000_000]`.

## Still open

- Borrowing parameters (waits on borrow syntax).
- Whether the long form `function.int32` is also valid.
