# Functions

Decided 2026-09-14. (Written before the `lang:` prefix was decided; read
`func` as `lang:func`, `var` as `lang:var`, `return` as `lang:return`.
Current spelling:)

```
lang:func.int32 'add' [lang:var.int32 'a', lang:var.int32 'b'] {
    lang:return ['a' + 'b'];
}
lang:var 'sum' := add[10, 20];              // this Limb: bare
lang:var 'big' := mathstuff::pow[2, 64];    // another Limb: Limb::path
```

## Calling (decided 2026-09-14)

A function is a member of a Limb. In the Limb where it is defined it is
called **bare**: `add[10, 20]`. From any other Limb it is reached through
the boundary: `mathstuff::pow[2, 64]`, `std::print.stdout[...]`. The name is
quoted only at its definition (`lang:func 'add'`), where it stands alone.

Bare words are unambiguous because nothing else is bare: keywords carry
`lang:`, other Limbs carry `::`, variables carry quotes. Consequently a call
(`add[...]`) and a list index (`'items'[3]`) are told apart by shape.

The operator words (`x`, `xx`, `mod`, `and`, `or`, `not`) are bare too, but
they never collide with a call: an operator sits *between* operands and is
never followed by `[`; a call is a word followed by `[`. `[mod[7, 3] mod 2]`
is legal and unambiguous. So they may be function names — zero reserved
words, no exception.

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

Arguments are comma-separated values in one `[ ]`, positionally matched to
the parameters. A call is a value and may appear inside any bracket:
`lang:var 'y' := [add[1, 2] x 3];`.

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

## Brackets

- `[ ]` — value slot(s).
- `{ }` — body.
- `( )` — arithmetic grouping only: `['a' + ('b' * 2)]`. This is why `[ ]`
  was chosen for values: parentheses stay free for math.

## Numbers

Digit grouping uses `_`, never `,`: `[1_000_000]`.

## Still open

- Borrowing parameters (waits on borrow syntax).
- Whether the long form `function.int32` is also valid.
