# Operators

Decided 2026-09-14.

| Operator | Meaning |
|---|---|
| `+` | add |
| `-` | subtract (and unary negation, assumed) |
| `x` | multiply |
| `xx` | power |
| `/` | divide |

`x`/`xx` carry over from legacy CyborgPL (the owner's design there). They are
unambiguous because user names are always quoted: `['x' x 2]` is the
variable `'x'` times two. Legacy's `xxx` (tetration) is **not** included.
`*` is not an operator.

`( )` groups: `['a' + ('b' x 2)]`.

## Not yet listed (open)

- Remainder / modulo.
- Comparison (`=`, `<`, `>`, …) — note `=` is already assignment in the
  typed form, so equality needs its own spelling.
- Boolean and/or/not — waits on whether a boolean type exists.
- Integer division vs. `/` on `int` (truncate? error on inexact?).
- Precedence: standard, or parentheses required when mixing levels.
