# Operators

Decided 2026-09-14.

| Operator | Meaning |
|---|---|
| `+` | add |
| `-` | subtract (and unary negation, assumed) |
| `x` | multiply |
| `xx` | power |
| `/` | divide |
| `mod` | remainder / modulo |
| `==` | equal |
| `!==` | not equal — built from `==`, same rule as `<==`/`>==` |
| `<`, `>` | less than, greater than |
| `<==`, `>==` | less-or-equal, greater-or-equal — built from `==`, because a lone `=` is assignment and must never appear inside a comparison |
| `and` | boolean and |
| `or` | boolean or |
| `not` | boolean not |

Word operators (`x`, `xx`, `mod`, `and`, `or`, `not`) are safe for the same
reason `x` is: user names are always quoted, so a bare word is never a name.

`x`/`xx` carry over from legacy CyborgPL (the owner's design there). They are
unambiguous because user names are always quoted: `['x' x 2]` is the
variable `'x'` times two. Legacy's `xxx` (tetration) is **not** included.
`*` is not an operator.

`( )` groups: `['a' + ('b' x 2)]`.

## Not yet listed (open)

- Integer division vs. `/` on `int` (truncate? error on inexact?).
- `mod` on negative operands (sign of the result).
- Precedence: standard, or parentheses required when mixing levels.
