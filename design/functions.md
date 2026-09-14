# Functions

Decided 2026-09-14. Current spelling:

```
func.int32 add [var.int32 'a', var.int32 'b'] {
    return ['a' + 'b'];
}
var 'sum' := [add[10, 20]];              // this Limb: bare
var 'big' := [mathstuff::pow[2, 64]];    // another Limb: Limb::path
```

## Calling (decided 2026-09-14)

A function is a member of a Limb. In the Limb where it is defined it is
called **bare**: `add[10, 20]`. From any other Limb it is reached through
the boundary: `mathstuff::pow[2, 64]`, `std::print.stdout[...]`.

**Function names are bare at the definition too** (decided 2026-09-14):
`func.int32 add [...]`. Quotes mean *variable*, and a function is not one.
A function name is therefore a plain identifier — no spaces or symbols, not
a keyword — as in every other language; variables keep the anything-goes
naming. (Quoting only the definition, as first written, was decoration: the
name had to be bare-legal to be callable anyway.)

A call (`add[...]`) and a list index (`'items'[3]`) are told apart by
shape; a keyword and a call are told apart by the reserved-word list (see
`syntax-sketch.md`).

The operator words (`x`, `xx`, `mod`, `and`, `or`, `not`) are bare too, but
they never collide with a call: an operator sits *between* operands and is
never followed by `[`; a call is a word followed by `[`. `[mod[7, 3] mod 2]`
is legal and unambiguous. So they may be function names; the operator words
are not reserved.

## Definition

```
func.int32 add [var.int32 'a', var.int32 'b'] {
    return ['a' + 'b'];
}
```

- The chain narrows `func` by its **return type**, the same way `var.int32`
  narrows `var`. `func name` with no type: Precog picks the best return type
  from every `return` in the body. No `return` at all: the function returns
  nothing.
- **Parameters are `var` declarations** the caller fills in, comma-separated
  in one `[ ]`. `var 'a'` with no type is legal: Precog infers the parameter's
  best type from every call site in the program.
- Body in `{ }`.

## Call

Arguments are comma-separated values in one `[ ]`, positionally matched to
the parameters. A call is a value, and a value position is always `[ ]` —
so storing a call's result wears an outer bracket, always:
`var 'sum' := [add[10, 20]];`, `var 'y' := [add[1, 2] x 3];`. A call as a
*statement* (`std::print.stdout[...]`, `shout[lend.mut 'u'];`) has no value
position, so no outer bracket. Decided 2026-09-14.

## Return

```
return ['a' + 'b'];
```

`[ ]` mandatory, as everywhere. Bare `return;` in a nothing-function.

## Entry point: `MAIN { }` (decided 2026-09-14)

```
MAIN {
    var 'sum' := [add[10, 20]];
    std::printu.stdout["sum: " 'sum' \n];
}
```

`MAIN` is the program's block — not a function. No `func`, no return type,
no `return`. When the block ends, the process exits 0; that is the one
implied thing in the language, and the one nobody has ever been surprised
by. Any other exit code is stated where it happens: `std::exit[1];`, typed
`uint8` because an exit status is 0–255 (Unix masks it to 8 bits; C's `int
main` is a lie by 24 bits).

`MAIN` is **unquoted** because it belongs to the language, not the user — a
quoted `'main'` would have been a reserved user name. It cannot be called:
it is not a function. It is the single root of Precog's whole-program
analysis and owns the program's top-level values.

A Limb with a `MAIN` is a program; without one, a library. Command-line
arguments come from the std Limb.

History: first decided as `func.int32 MAIN` with a mandatory `return`, then
simplified the same day — the manual `return 0` only ever existed because C
made `main` an ordinary function, and C99 had to add the implicit 0 anyway.

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
