# Operators

Decided 2026-09-14.

**`div` and `</==` / `>/==`** (owner, 2026-09-29): division is the word
`div`, and `/` is no longer an operator. Freed from division, the slash
reads as "or", so less-or-equal is `</==` ("less / equal") and
greater-or-equal `>/==`. Subtraction and negation both stay `-`: the
owner first made negation `neg` (`neg'x'`), then went back to `-` once
it meant writing minus five as `neg 5`. The old spellings are refused
with an error naming the new ones.

| Operator | Meaning |
|---|---|
| `+` | add |
| `-` | subtract (and unary negation, assumed) |
| `x` | multiply |
| `xx` | power |
| `div` | divide (was `/` until 2026-09-29) |
| `mod` | remainder / modulo |
| `==` | equal |
| `!==` | not equal — built from `==` |
| `<`, `>` | less than, greater than |
| `</==`, `>/==` | less-or-equal, greater-or-equal — the slash reads as "or" ("less / equal"), and `==` because a lone `=` is assignment and must never appear inside a comparison (was `<==` / `>==` until 2026-09-29) |
| `and` | boolean and |
| `or` | boolean or |
| `not` | boolean not |

Word operators (`x`, `xx`, `div`, `mod`, `and`, `or`, `not`) are safe for the same
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
in full: `change 'i' = 'i' + 1;`. Precog emits the same instruction either way;
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
| 3 | `x` `div` `mod` | `x` chains freely; `div` or `mod` beside `x`, `div`, or `mod` **requires parentheses** (`6 div 2 x 3` has no settled reading); `mod` beside *any* operator requires parentheses (it has no mathematical notation) |
| 4 | `+` `-` | left-associative |
| 5 | `==` `!==` `<` `>` `</==` `>/==` | never chained (`1 < 2 < 3` is an error); never beside `not` |
| 6 | `not` | applies to one operand or a parenthesis |
| 7 | `and` | ∧ before ∨, as in logic |
| 8 | `or` | |

**Bitwise** (decided 2026-09-21): `and.bit`, `or.bit`, `not.bit`, `xor`, `shl`, `shr`. A bitwise `and` is the boolean `and` applied to every bit position at once (`1100 and.bit 1010 = 1000`: a column keeps a 1 only if both have one), so the spelling narrows the boolean word with `.bit` rather than inventing a second word — C's `&` vs `&&` confusion is what a separate spelling invites; `xor` has no boolean twin so it stands alone; shifts extend nothing. Integers only; the result is the operand's type; `shl` past the width or into overflow is a check site. No precedence: mathematics has no reading for a bitwise operator beside an arithmetic one, so parentheses whenever one meets another operator. **Wrapping arithmetic** (decided 2026-09-20): `+~`, `-~`, `x~`, `xx~` — "plus, but modulo the type's size" — always defined, never a check, nothing to `nocheck`; same precedence as the plain forms. For hashes, ring counters, PRNGs, where wrapping *is* the algorithm; plain `+ - x xx` stay overflow-checked, exact and checked where they land (2026-09-23, below). Owner chose the tilde ("wraps around") over Zig's `+%` (`%` reads as percent — and `mod` is a word here for that reason) and Swift's `&+` (`&` reads as borrow to a Rust hire). The width is the operand's type — any operand whose type is known, see below. **`div` on integers floors and `mod` is the mathematician's** (decided 2026-09-20): `-7 div 2 = -4`, `-7 mod 2 = 1` — the result of `mod` has the divisor's sign, so `x mod 12` is always 0–11 and a clock, a hash, a ring buffer never go negative (the classic C `%` bug). The identity `a == (a div b) x b + (a mod b)` holds. Hardware truncation (`-7 div 2 = -3`, `-7 mod 2 = -1`) is std's, by name — `std::div.trunc[a, b]`, `std::mod.trunc[a, b]` — for matching a C library bit for bit. Cost: a few one-cycle instructions beside a 10–40-cycle divide, and only where Precog cannot prove a sign (unsigned types, counters, positions, lengths need nothing); the fix-up is the operator's meaning, not a check, so it is neither `nocheck`-able nor an error in the final build. Owner asked about performance and whether it ships; chose floor and std for truncation. `++` joins text (decided 2026-09-19): below `+ -`, above comparisons, left-associative; at least one side text, the other rendered in stored form. So `1 + 2 x 3` = 7, `2 x 3 xx 2` = 18, `'a' + 1 > 'b' - 2 and not ('a' == 'b')` reads as expected, and `6 div 2 x 3`, `10 mod 3 + 1`, `not 1 == 2`, `'i' mod 2 == 0` are errors that say "parenthesise" (`('i' mod 2) == 0`).

## A shift's count (owner, 2026-09-23: a)

The count must be **0 to width − 1**, for `shl` and `shr` alike;
anything else is a check site. Shifting every bit out of an 8-bit
number is a mistake — the answer is always 0, and nobody needs a shift
to get 0 — and a negative count is never written on purpose, since the
two directions are already two words: the owner's reading, "why don't
they just use minus?", is the argument — `'v' shr (-1)` means `'v' shl
1`, so a negative count is a variable that came out wrong. C and LLVM
leave both cases undefined, which is the other reason to check rather
than define them: a count Precog cannot bound becomes a guardrail, not
a silent 0. Proposed alternatives: `shr` by the width or more is
defined as 0 and only a negative count is checked; that for `shr` and
the strict rule for `shl`.

**`shr` on a signed value is arithmetic** — it keeps the sign, so
`-100 shr 2` is `-25`. Both engines already did this; it had never been
written down.

## Where a shift's width comes from (owner, 2026-09-23: a)

`shl` can push bits off the left-hand end, so the compiler must know
how wide the number is. **Any operand whose type it can work out is
enough** — it does not have to be a declared name:

```
'v': uint32 = 3;
'a': uint32 = 2 shl 3;              # 16 — the literal takes the uint32 of its context
'b': uint32 = ('v' mod 16) shl 2;   # 12 — the expression is plainly a uint32
```

Same for `not.bit` and the wrapping operators. A literal takes its
width from its context, as a literal takes its type from its context
everywhere else in the language. Proposed alternatives: the left
operand must be a name or a `to.T[…]`, so the width is always visible
in the operation itself; or that rule for bare literals only. Owner:
the compiler can work it out, so it should.

**A list literal's items take the element type** (owner, 2026-09-29: a):
`'xs': list[uint8] = [0 -~ 1,];` holds 255, exactly as `'x': uint8 = 0 -~
1;` does, since each item lands in a `uint8` slot. So the "construction"
below is a struct's or a call's, not a list literal. Raised by check
claude; proposed alternative (b): the size stops at the brackets, so an
item needing a size writes it (`[to.uint8[0] -~ 1,]`). `list.append` is a
call, so its argument still gets no context.

**Context stops at a call** (2026-09-23): the type a value is stored
into reaches the arithmetic that produces it, but not through a call, a
conversion or a construction — what `g[…]` *returns* says nothing about
the arithmetic inside its arguments. So `'v': uint8 = g[0 > (-1) -
'x'];` is fine when `'x'` is an `int64`, while `'w': uint32 = 2 shl
3;` still takes its width from the declaration.

It stops at a **comparison** as well: a `bool` holds the comparison's
answer, never its operands, which take their width from each other by
the rule above. So `'b': bool = (5 -~ 'w') > 1;` is a `uint32`
subtraction when `'w'` is a `uint32`. And the other side does size it: in
`'n' > (0 -~ 1)` with `'n'` a `uint64`, the literal group wraps in
`uint64`.

The sentence above — "the compiler knows every width; the oracle needs
the left operand declared" — was always about the oracle's limit, not
the language's rule. CCA had read it as a rule and refused both lines;
it is the one that changes. Found by the generator on its first run: it
was two-thirds of every disagreement until check claude steered around
it.

## Plain math is exact; it is checked where it lands (owner, 2026-09-23)

```
'small': int16 = 5;
'big': int64 = (20000 + 20000) + 'small';   # 40005: fits int64, fine
'tiny': int8 = (100 + 100) - 150;           # 50: the 200 in between is fine
'bad': int8 = 100 + 100;                    # 200 does not fit int8: error
```

`+ - x xx` on integers give the true answer, and the check is where the
value lands: a typed declaration, a `change`, an argument (the parameter's
type), a return (the return type). A comparison compares true values,
and a `:=` takes a type that holds its true value. You never get a
wrong number, and never an error for an answer that fits where it is
going. In between, the one limit is the machine's 64 bits: a value no
64-bit type holds is a check site (`'a' x 'a'` on two huge `int64`s),
which Precog removes wherever it proves the values smaller. Bare
numbers alone are worked out exactly at compile time. Precog still uses
narrow sizes where it proves they fit, for SIMD especially, since the
answer is the same.

**Wrapping is the exception**: `+~ -~ x~ xx~` wrap at the size of their
values, never at where the result goes, so a hash gives the same number
on any line (`'x' +~ 'x'` on a `uint32` wraps at 32 bits even when
stored into a `uint64`). **When the values mix sizes, you say which**
(owner, 2026-09-23: b): `('a' + 'b') +~ 1` with an `int8` and an
`int16` is an error until written `to.int16['a' + 'b'] +~ 1`, and so is
`'a' +~ 'b'`. Wrapping exists for hashes and counters, where the size
is the algorithm, so it is written and never guessed. Proposed
alternatives: the bigger size; the left operand's (the oracle's old
placeholder).

**A negative exponent is a check site** (owner, 2026-09-24: a), for `xx`
and `xx~` alike: `2 xx -1` is a half, which no integer holds, and a
fraction is not something wrapping answers. Precog sees the whole
program and proves the exponent never negative, the working build
checks where it cannot, and the final build refuses an unproven one.
Proposed alternatives: a fixed answer such as 0, so `xx~` stays
check-free; an exponent that must be unsigned. For `xx~` only the base's
size counts: the exponent is a count, like a shift's, not a value being
wrapped (the oracle's reading, not ruled).

How it was reached: check claude asked which size a group of bare
numbers takes when the name beside it (`int16`) and the place it is
stored (`int64`) disagree. Proposed: (a) the name beside decides; (b)
the stored size decides; (c) split, plain by the stored size, wrapping
by the values; (d) bare numbers exact, a mix of sizes done at the
bigger. The owner first reasoned "the math is done inside `'big'`" (b),
then asked what is best for performance and safety. The answer was
this: every option before it either errors on answers that fit (a, and
b in the middle of a narrower line) or can move a wrap (b). On today's
processors a 16-bit and a 64-bit add cost the same, and the checks are
what cost, and this has the fewest. Owner: "yes, that one". It also
answers mixed sizes (`int16 + int64`), which nothing had decided.

## `bin16` / `bin32` math rounds after every step (owner, 2026-09-24: a)

```
'x': bin32 = 16777216.0;
'y': bin32 = 'x' + 1.0 + 1.0;      # 16777216: each + rounds back at bin32
```

Each operation on `bin16` or `bin32` values gives the IEEE result at their
width (nearest-even), as C and the processor do. A bare literal takes the
width of the typed value beside it, and the place a result is stored never
changes it. A result that rounds past the largest is a check site. Anyone
who needs the exact sum chooses a bigger or exact type (`bin64`, an
integer, `std::uenr`): "the programmer could've chose a better one". This is
the opposite of plain integer math, which is exact because there 16- and
64-bit cost the same. For bins, 64-bit intermediates would halve SIMD lanes
and could not run on Metal, which has no 64-bit float. Proposed
alternative: 64-bit intermediates rounded once where stored.

## `and` / `or` always work out both sides (owner, 2026-09-25: b)

```
'ok' := check_name['f'] and check_age['f'];     # both run, left then right
if 'i' </== list.length[lend 'xs'] {             # a check that protects the next
    if 'xs'['i'] == 5 { … }                 # step is written as two `if`s
}
```

Nothing written on a line is skipped: `and` and `or` evaluate the left
side, then the right, even when the left already decides the answer.
The one-line guard `'i' </== … and 'xs'['i'] == 5` is therefore not a
guard. The read runs when `'i'` is past the end, so it stops the working
build and the final build refuses it. It is safe either way, never
silent. The owner's reason: the code does exactly what it says ("so the
code isn't lying"). Proposed alternatives: stop early, as C, Rust, Python,
Go and JavaScript do, so the one-line guard works and Precog narrows the
right side by the left (recommended by check claude and design claude);
run both and refuse such lines at compile time.

**`and.then` / `or.else` stop once the answer is known** (owner, same
day), for when the second half should depend on the first:

```
if 'i' </== list.length[lend 'xs'] and.then 'xs'['i'] == 5 { … }
if 'n' == 0 or.else 100 div 'n' < 2 { … }
```

A variant of `and`/`or` by the dot rule, like `and.bit`. It reads as the
English: "A, and then B" (only if A held) and "A, or else B" (only if A
failed), the spelling Ada uses (`and then` / `or else`). Same precedence as
`and` / `or`. Precog narrows the right side by the left, so the read and
the division above are proven. Names considered: `and.guard`, `and.early`,
`and.short`. Handling it with `otherwise` was asked about and does not
apply: a list read out of range or a division by zero is a check site,
not a failure. A failing `list.get` was floated for the list case and not
chosen.

## Not yet listed (open)

(Integer `div` (then `/`) and `mod` were decided 2026-09-20: floor division, the
mathematician's `mod`; see the bitwise and wrapping paragraph above.)

## Custom operators — `op` (decided 2026-09-20)

```
struct Vec { x: bin64  y: bin64 }
op <+> ['a': Vec, 'b': Vec] -> Vec { return Vec[x = 'a'.x + 'b'.x, y = 'a'.y + 'b'.y]; }
op dot ['a': Vec, 'b': Vec] -> bin64 { return 'a'.x x 'b'.x + 'a'.y x 'b'.y; }
op **  [give 'a': Vec, 'b': bin64] -> Vec { … return give 'a'; }

'w' := 'u' <+> 'v';                 # 'u' and 'v' both still yours
'd' := ('u' <+> 'v') dot 'v';       # parenthesise when operators meet
's' := give 'u' ** 10;              # this op takes 'a' by value; the caller says so
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

### From another Limb (owner, 2026-09-23: a)

A `pub op` is usable by every Limb that depends on the one that wrote
it, with nothing written: an operator has no place for a path, so
`'u' dot 'v'` finds the `dot` written for what `'u'` and `'v'` are. Any
Limb may share an operator for any types, the language's own included
(`op ~= ['a': bin64, 'b': bin64] -> bool` for "equal within a margin").
**Two meanings of one operator for the same types is a compile error**
(owner: "shouldn't two meanings of `~=` just err?"), whether they come
from two Limbs or from a Limb and your own code. With no path to pick
one, the fix is to drop a Limb, or copy one operator into your own
code and edit it. Proposed alternative (design claude's
recommendation): only the Limb that owns one of the operand types may
share an operator, as in Rust, so two can never clash. That would stop
a Limb from sharing `~=` or `+|` for the language's own types.

Oracle: `examples/operators.cpls`, thirteen reject tests. It runs one
Limb, so an operator is found by name across the whole program.
