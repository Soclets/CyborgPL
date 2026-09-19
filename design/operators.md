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
unambiguous because user names are always quoted: `'x' x 2` is the
variable `'x'` times two. Legacy's `xxx` (tetration) is **not** included.
`*` is not an operator.

`( )` groups: `'a' + ('b' x 2)`.

**`xx` chains follow mathematics** (decided 2026-09-14): `2 xx 3 xx 2` is
`2 xx (3 xx 2)` = 512, a power tower read from the top. Parentheses always
override, even against convention: `(2 xx 3) xx 2` = 64. The other
operators chain left as usual.

## No increment or compound assignment (decided 2026-09-14)

There is no `++`, `--`, `+=`, or any other shorthand. Adding one is written
in full: `'i' = 'i' + 1;`. Precog emits the same instruction either way;
the shorthand would exist only for the reader, and the owner chose not to
have it.

## Precedence (decided 2026-09-14)

**Whatever mathematics says is right. Where mathematics has no settled
answer, parentheses are required. Parentheses always override, even against
convention.**

Tightest to loosest:

| Level | Operators | Rule |
|---|---|---|
| 1 | `xx` | right-associative: `2 xx 3 xx 2` = 512 |
| 2 | unary `-` | below `xx`: `-2 xx 2` = −4; `(-2) xx 2` = 4 |
| 3 | `x` `/` `mod` | `x` chains freely; `/` or `mod` beside `x`, `/`, or `mod` **requires parentheses** (`6 / 2 x 3` has no settled reading); `mod` beside *any* operator requires parentheses (it has no mathematical notation) |
| 4 | `+` `-` | left-associative |
| 5 | `==` `!==` `<` `>` `<==` `>==` | never chained (`1 < 2 < 3` is an error); never beside `not` |
| 6 | `not` | applies to one operand or a parenthesis |
| 7 | `and` | ∧ before ∨, as in logic |
| 8 | `or` | |

So `1 + 2 x 3` = 7, `2 x 3 xx 2` = 18, `'a' + 1 > 'b' - 2 and not ('a' == 'b')` reads as expected, and `6 / 2 x 3`, `10 mod 3 + 1`, `not 1 == 2`, `'i' mod 2 == 0` are errors that say "parenthesise" (`('i' mod 2) == 0`).

## Not yet listed (open)

- Integer division vs. `/` on `int` (truncate? error on inexact?).
- `mod` on negative operands (sign of the result).

