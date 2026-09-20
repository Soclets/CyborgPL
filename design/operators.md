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

**Wrapping arithmetic** (decided 2026-09-20): `+~`, `-~`, `x~`, `xx~` — "plus, but modulo the type's size" — always defined, never a check, nothing to `nocheck`; same precedence as the plain forms. For hashes, ring counters, PRNGs, where wrapping *is* the algorithm; plain `+ - x xx` stay overflow-checked. Owner chose the tilde ("wraps around") over Zig's `+%` (`%` reads as percent — and `mod` is a word here for that reason) and Swift's `&+` (`&` reads as borrow to a Rust hire). The width is the operand's type (the compiler knows every width; the oracle needs the left operand declared, e.g. `'h': uint32`). **`/` on integers floors and `mod` is the mathematician's** (decided 2026-09-20): `-7 / 2 = -4`, `-7 mod 2 = 1` — the result of `mod` has the divisor's sign, so `x mod 12` is always 0–11 and a clock, a hash, a ring buffer never go negative (the classic C `%` bug). The identity `a == (a / b) x b + (a mod b)` holds. Hardware truncation (`-7 / 2 = -3`, `-7 mod 2 = -1`) is std's, by name — `std::div.trunc[a, b]`, `std::mod.trunc[a, b]` — for matching a C library bit for bit. Cost: a few one-cycle instructions beside a 10–40-cycle divide, and only where Precog cannot prove a sign (unsigned types, counters, positions, lengths need nothing); the fix-up is the operator's meaning, not a check, so it is neither `nocheck`-able nor an error in the final build. Owner asked about performance and whether it ships; chose floor and std for truncation. `++` joins text (decided 2026-09-19): below `+ -`, above comparisons, left-associative; at least one side text, the other rendered in stored form. So `1 + 2 x 3` = 7, `2 x 3 xx 2` = 18, `'a' + 1 > 'b' - 2 and not ('a' == 'b')` reads as expected, and `6 / 2 x 3`, `10 mod 3 + 1`, `not 1 == 2`, `'i' mod 2 == 0` are errors that say "parenthesise" (`('i' mod 2) == 0`).

## Not yet listed (open)

- Integer division vs. `/` on `int` (truncate? error on inexact?).
- `mod` on negative operands (sign of the result).

## Custom operators — `op` (decided 2026-09-20)

```
struct Vec { x: bin64  y: bin64 }
op <+> ['a': Vec, 'b': Vec] -> Vec { return Vec[x = 'a'.x + 'b'.x, y = 'a'.y + 'b'.y]; }
op dot ['a': Vec, 'b': Vec] -> bin64 { return 'a'.x x 'b'.x + 'a'.y x 'b'.y; }
op **  [give 'a': Vec, 'b': bin64] -> Vec { … return give 'a'; }

let 'w' := 'u' <+> 'v';                 # 'u' and 'v' both still yours
let 'd' := ('u' <+> 'v') dot 'v';       # parenthesise when operators meet
let 's' := give 'u' ** 10;              # this op takes 'a' by value; the caller says so
```

Four questions, one at a time:

1. **Words and glyphs** (owner: "glyphs too", over Claude's words-only).
   A word operator is a bare non-keyword word between operands — the
   mechanism `x`, `mod`, `and` already use, possible only because names
   are quoted.
2. **Which characters, how long.** A glyph operator is any run of
   `! % & * + - / < > = ? @ ^ | ~` — the characters not taken by the
   grammar (`$` is the shadow mark, `:` `.` and the brackets are
   structure). **A maximal run is one operator**; a space separates two:
   `'a' <+> -'b'` is `<+>` then a negation, `'a' <+>-'b'` is the operator
   `<+>-`. Claude proposed a three-character cap; owner: "isn't `<+>-` an
   operator? if they wanted negative, they could put that inside `b`" —
   no cap. The language's own operators and its deliberate near-misses
   (`!=`, `<=`, `>=`, `--`, `+=`, `-=`) cannot be redefined; `*`, `^`,
   `%`, `&`, `|`, `!` are free to declare, and undeclared they still
   point at `x`, `xx`, `mod`, `and`/`or`/`not`.
3. **No precedence** (owner: "Require"). A custom operator next to any
   other operator is a compile error until parenthesised — the `mod`
   rule: where there is no settled reading, parens. Nobody memorises a
   library's table.
4. **Operands are lent by default; `give` on an operand takes it by value,
   and then the caller writes `give` too** (owner: "middle", between
   always-lent and Rust's by-value). Rust's `a + b` moves both operands,
   which is why every numeric crate writes four `impl`s per operator;
   always-lent reads like arithmetic but cannot reuse a dead operand's
   buffer (`s = s + "x"` in a loop). The middle keeps `'u' <+> 'v'` for
   the common case and lets an `op` say `give 'a'` when it wants the
   memory. Precog may elide the copy in the lent case when the operand is
   dead; the `give` form guarantees it.

Rules that follow: exactly two operands; `-> T` required; an operator
cannot fail (make it a `fn`); it is never called with `[…]`; `not`, `and`,
`or` cannot be redefined; the language's other operators may be **defined
for a user type** (`op + ['a': Vec, 'b': Vec]`, decided 2026-09-20 with
interfaces) but never changed for the language's own types. Prefix (one-operand) custom
operators are not offered.

Oracle: `examples/operators.cpls`, thirteen reject tests. It resolves an
operator by name across the whole program (visibility not modelled for
operators — open).
