# Operators

> Revised 2026-10-07: code here is in the new syntax; an old spelling in the prose is history. The old → new table is `revision-2026-10-07.md`.

Decided 2026-09-14.

**`div` and `</==` / `>/==`** (owner, 2026-09-29): division is the word
`div`, and `/` is no longer an operator. Freed from division, the slash
reads as "or", so less-or-equal is `</==` ("less / equal") and
greater-or-equal `>/==`. Subtraction and negation both stay `-`: the
owner first made negation `neg` (`neg'x'`), then went back to `-` once
it meant writing minus five as `neg 5`. The old spellings are refused
with an error naming the new ones.

**Every division says how it rounds** (owner, 2026-10-02: "Make bare
`div` illegal", then b, a, a, b): `div.floor` (down), `div.ceil` (up),
`div.trunc` (toward zero) and `div.near` (to the nearest, a tie to even),
on **every** number family; a bare `div` is refused, naming the four.
`mod` follows one for one, `mod.floor` … `mod.near`, each what its `div`
leaves over: `a == (a div.X b) times b plus (a mod.X b)`. On integers every
quotient is whole (`7 div.near 2` is 4, `5 div.near 2` is 2). On a bin,
`brain16` or `deci`, `div.near` is ordinary division, rounded to the
nearest value the type holds (`7.5 div.near 2.0` is 3.75), and
`floor`/`ceil`/`trunc` give a whole quotient (`7.5 div.floor 2.0` is 3.0,
Python's `//`); so `mod.near` is refused there, since it leaves nothing
over (design claude's slip: it had first called it IEEE's `remainder()`).
Proposed for bins: `div.near` naming the rounding (chosen), `div.real`,
`div.frac`. `examples/div_rounding.cy`.

**Current spelling (2026-10-16): symbols only where they read correctly, and then they are the good
practice.** `+` (`plus`), `-` (`minus`), `x` (`times`), `xx` (std's `pow`), `<` (`less_than`), `>`
(`greater_than`) and a leading `-` (`-5`, `-'x'`, `negate`); each word stays a valid spelling of the same
operator, and **a variation is always the word**: `plus.wrap`, `times.ieee`, `div.floor`, `negate.ieee`. Words
only, their symbols refused as old spellings naming the word: `equal_to`, `not_equal`, `at_most`, `at_least`
(`==`, `!=`, `<=`, `>=`); `div.…` (`/`); `mod.…` (`%`); `and`, `and.then`, `or.else` (`&`, `&&`, `||`);
`concat` (`++`). No compound changes: `change 'i' = 'i' + 2;`. A type defines its own with `op plus […]`; an
operator as a value is `op plus`. `DECISIONS.md` 2026-10-16 has the whole list.

History: from 2026-10-02/03 to 2026-10-14 the operators were words only (`plus`, `minus`, `times`) and the
symbols were refused; 2026-10-14 brought the symbols back as spellings of the words, and 2026-10-16 narrowed
them to the list above. Passages below keep whatever spelling their ruling's day used.

| Operator | Meaning |
|---|---|
| `plus` | add (`+` until 2026-10-03) |
| `minus` | subtract (`-` until 2026-10-03); a negative keeps `-`: `-5`, `-'x'` |
| `times` | multiply (`x` until 2026-10-03) |
| `plus.wrap` `minus.wrap` `times.wrap` | wrapping (`+~ -~ x~` until 2026-10-03) |
| `div.floor.wrap` `div.ceil.wrap` `div.trunc.wrap` `div.near.wrap` | the true quotient wrapped at the type's width, as the hardware does for the smallest value divided by -1 (2026-10-03); `mod` never overflows, so there is no `mod.X.wrap` |
| `pow`, `pow.wrap` | power: std's operators since 2026-10-03 (the language's `xx`, `xx~` before); bare or `std::pow`; above a negative, right pair first, as `xx` was (`-2 pow 2` is -4, `2 pow 3 pow 2` is 512) |
| `div.floor` `div.ceil` `div.trunc` `div.near` | divide, rounding as named (`div` until 2026-10-02, `/` until 2026-09-29) |
| `mod.floor` `mod.ceil` `mod.trunc` `mod.near` | what the same `div` leaves over |
| `plus.ieee` `minus.ieee` `times.ieee` `pow.ieee` `div.floor.ieee` `div.ceil.ieee` `div.trunc.ieee` `div.near.ieee` `mod.floor.ieee` `mod.ceil.ieee` `mod.trunc.ieee`, and `neg.ieee 'x'` | IEEE 754's own answer, keeping a `-0`; on a bin or `brain16` only. Every other bin or `brain16` result is never `-0`: a zero times a negative, `-'x'` of a zero, a product too small to hold are all `0` (owner, 2026-10-07: "`a`", then "yea" to the family and to `neg.ieee`; `-.ieee 'x'` "defeats the purpose of a symbol"; `mod` and `pow` too, "yea."). `examples/zero_everywhere.cy` |
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
unambiguous because user names are always quoted: `'x' times 2` is the
variable `'x'` times two. Legacy's `xxx` (tetration) is **not** included.
`*` is not an operator.

`( )` groups: `'a' plus ('b' times 2)`.

**`xx` chains follow mathematics** (decided 2026-09-14): `2 pow 3 pow 2` is
`2 pow (3 pow 2)` = 512, a power tower read from the top. Parentheses always
override, even against convention: `(2 pow 3) pow 2` = 64. The other
operators chain left as usual.

## No compound changes (owner, 2026-10-16: (a))

A change is always written out: `change 'i' = 'i' + 1;`, `change 'total' = 'total' x 2;`. From 2026-10-14
to 16 the symbols `'i'++;`, `'i'--;`, `+=`, `-=`, `*=`, `/=`, `%=` and `**=` spelled these (owner then: "we
add it as an alt"); asked "what the fuck is `+=`?", offered (a) none, (b) only `++` / `--`, (c) all, the owner
took (a), as design claude recommended: they do not read as what they do, and `change` already marks every
change. Each is now an old spelling, refused naming the `change` to write.

## Precedence (decided 2026-09-14)

**Whatever mathematics says is right. Where mathematics has no settled
answer, parentheses are required. Parentheses always override, even against
convention.**

Tightest to loosest:

| Level | Operators | Rule |
|---|---|---|
| 1 | `xx` | right-associative: `2 pow 3 pow 2` = 512 |
| 2 | unary `-` | below `xx`: `-2 pow 2` = −4; `(-2) pow 2` = 4 |
| 3 | `x` `div.X` `mod.X` | `x` chains freely; a `div.X` or `mod.X` beside `x`, `div.X`, or `mod.X` **requires parentheses** (`6 div.floor 2 times 3` has no settled reading); a `mod.X` beside *any* operator requires parentheses (it has no mathematical notation) |
| 4 | `+` `-` | left-associative |
| 5 | `==` `!==` `<` `>` `</==` `>/==` | never chained (`1 < 2 < 3` is an error); never beside `not` |
| 6 | `not` | applies to one operand or a parenthesis |
| 7 | `and` | ∧ before ∨, as in logic |
| 8 | `or` | |

**Bitwise** (decided 2026-09-21): `and.bit`, `or.bit`, `not.bit`, `xor`, `shl`, `shr`. A bitwise `and` is the boolean `and` applied to every bit position at once (`1100 and.bit 1010 = 1000`: a column keeps a 1 only if both have one), so the spelling narrows the boolean word with `.bit` rather than inventing a second word — C's `&` vs `&&` confusion is what a separate spelling invites; `xor` has no boolean twin so it stands alone; shifts extend nothing. Integers only; the result is the operand's type; `shl` past the width or into overflow is a check site. No precedence: mathematics has no reading for a bitwise operator beside an arithmetic one, so parentheses whenever one meets another operator. **Wrapping arithmetic** (decided 2026-09-20): `+~`, `-~`, `x~`, `xx~` — "plus, but modulo the type's size" — always defined, never a check, nothing to `no_check`; same precedence as the plain forms. For hashes, ring counters, PRNGs, where wrapping *is* the algorithm; plain `+ - x xx` stay overflow-checked, exact and checked where they land (2026-09-23, below). Owner chose the tilde ("wraps around") over Zig's `+%` (`%` reads as percent — and `mod` is a word here for that reason) and Swift's `&+` (`&` reads as borrow to a Rust hire). The width is the operand's type — any operand whose type is known, see below. **`div` on integers floors and `mod` is the mathematician's** (decided 2026-09-20): `-7 div 2 = -4`, `-7 mod 2 = 1` — the result of `mod` has the divisor's sign, so `x mod 12` is always 0–11 and a clock, a hash, a ring buffer never go negative (the classic C `%` bug). The identity `a == (a div b) x b + (a mod b)` holds. Hardware truncation (`-7 div 2 = -3`, `-7 mod 2 = -1`) is by name, for matching a C library bit for bit: **`a div.trunc b`, `a mod.trunc b`, operators in the language** (owner, 2026-10-02: "I think `div.trunc` should be in base", then "`a`", an operator narrowing `div` as `.bit` narrows `and`, over a built-in `div.trunc[a, b]`; and "yea", `mod.trunc` with it), at `div`'s precedence and under `mod`'s parenthesis rule. Until then they were std's, `std::div.trunc[a, b]`, now refused naming the operator. `div.trunc` takes integers; `mod.trunc` integers, bins and `brain16`. Cost: a few one-cycle instructions beside a 10–40-cycle divide, and only where the prover cannot prove a sign (unsigned types, counters, positions, lengths need nothing); the fix-up is the operator's meaning, not a check, so it is neither `no_check`-able nor an error in the final build. Owner asked about performance and whether it ships; chose floor and std for truncation. **The same on a bin and `brain16`** (owner, 2026-10-02: a, compiler claude's question): `-370.0 mod 360.0` is `350`, `7.5 mod -2.0` is `-0.5`; C's `fmod` is `mod.trunc` there too. The oracle had given the dividend's sign by accident (Rust's `%`). It costs a compare and an add beside the `fmod` routine, none where the prover proves both signs. A tiny negative over a large divisor would round up to the divisor itself (`-1e-20 mod 360.0` gives `360.0` in Python); here it is `0`, so the answer is always below the divisor (owner, 2026-10-02: b, "basically none" of a cost: one compare). **`concat` joins text, and `++` is its symbol** (owner, 2026-10-14: "(a), `concat`"; `"Hello, " concat 'name'`). `++` joins text (decided 2026-09-19): below `+ -`, above comparisons, left-associative; at least one side text, the other rendered in stored form. So `1 + 2 x 3` = 7, `2 x 3 xx 2` = 18, `'a' + 1 > 'b' - 2 and ! ('a' == 'b')` reads as expected, and `6 div 2 x 3`, `10 mod 3 + 1`, `not 1 == 2`, `'i' mod 2 == 0` are errors that say "parenthesise" (`('i' mod 2) == 0`).

## A shift's count (owner, 2026-09-23: a)

The count must be **0 to width − 1**, for `shl` and `shr` alike;
anything else is a check site. Shifting every bit out of an 8-bit
number is a mistake — the answer is always 0, and nobody needs a shift
to get 0 — and a negative count is never written on purpose, since the
two directions are already two words: the owner's reading, "why don't
they just use minus?", is the argument — `'v' shr (-1)` means `'v' shl
1`, so a negative count is a variable that came out wrong. C and LLVM
leave both cases undefined, which is the other reason to check rather
than define them: a count the prover cannot bound becomes a guardrail, not
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
let $v: int.unsign.32 := 3;
let $a: int.unsign.32 := 2 shl 3;              # 16 — the literal takes the u32 of its context
let $b: int.unsign.32 := ($v mod.floor 16) shl 2;   # 12 — the expression is plainly a u32
```

Same for `not.bit` and the wrapping operators. A literal takes its
width from its context, as a literal takes its type from its context
everywhere else in the language. Proposed alternatives: the left
operand must be a name or a `to.T[…]`, so the width is always visible
in the operation itself; or that rule for bare literals only. Owner:
the compiler can work it out, so it should.

**A list literal's items take the element type** (owner, 2026-09-29: a):
`let 'xs': list[u8] = [0 minus.wrap 1,];` holds 255, exactly as `let 'x': uint8 = 0 -~
1;` does, since each item lands in a `uint8` slot. So the "construction"
below is a struct's or a call's, not a list literal. Raised by check
claude; proposed alternative (b): the size stops at the brackets, so an
item needing a size writes it (`[to.u8[0] minus.wrap 1,]`). **An argument takes its
parameter's type the same way** (owner, same day: a): `show[0 minus.wrap 1]` into a
`u8` parameter wraps at 8 bits, and `list.append`'s value takes the
element type. Proposed alternative: an argument has no context, and says
its size itself.

**Context stops at a call** (2026-09-23): the type a value is stored
into reaches the arithmetic that produces it, but not through a call (an
argument takes its parameter's type instead, 2026-09-29), a
conversion or a construction — what `g[…]` *returns* says nothing about
the arithmetic inside its arguments. So `'v': uint8 = g[0 > (-1) -
'x'];` is fine when `'x'` is an `int64`, while `'w': uint32 = 2 shl
3;` still takes its width from the declaration.

It stops at a **comparison** as well: a `bool` holds the comparison's
answer, never its operands, which take their width from each other by
the rule above. So `let 'b': bool = (5 minus.wrap 'w') > 1;` is a `u32`
subtraction when `'w'` is a `u32`. And the other side does size it: in
`'n' > (0 minus.wrap 1)` with `'n'` a `u64`, the literal group wraps in
`u64`.

The sentence above — "the compiler knows every width; the oracle needs
the left operand declared" — was always about the oracle's limit, not
the language's rule. kayo had read it as a rule and refused both lines;
it is the one that changes. Found by the generator on its first run: it
was two-thirds of every disagreement until check claude steered around
it.

## Plain math is exact; it is checked where it lands (owner, 2026-09-23)

```
let $small: int.sign.16 := 5;
let $big: int.sign.64 := (20000 plus 20000) plus $small;   # 40005: fits i64, fine
let $tiny: int.sign.8 := (100 plus 100) minus 150;           # 50: the 200 in between is fine
let $bad: int.sign.8 := 100 plus 100;                    # 200 does not fit i8: error
```

`plus minus times` (and std's `pow`) on integers give the true answer, and so does every `div.X`
and `mod.X` (owner, 2026-10-02: a; the one case is the smallest value
divided by -1, `-128 div.trunc -1` on `i8`s, which is 128), and the
check is where the value lands: a typed declaration, a `change`, an argument (the parameter's
type), a return (the return type). A comparison compares true values,
and a `:=` takes a type that holds its true value. You never get a
wrong number, and never an error for an answer that fits where it is
going. In between, the one limit is the machine's 64 bits, or the widest
operand's width when that is wider (owner, 2026-10-03: two
`!std.uint256`s work at 256 bits, two `i128`s at 128): a value no type
of that width holds is a check site (`'a' times 'a'` on two huge `i64`s),
which the prover removes wherever it proves the values smaller. Bare
numbers alone are worked out exactly at compile time. the prover still uses
narrow sizes where it proves they fit, for SIMD especially, since the
answer is the same.

**Wrapping is the exception**: `plus.wrap minus.wrap times.wrap` (and std's `pow.wrap`) wrap at the size of their
values, never at where the result goes, so a hash gives the same number
on any line (`'x' plus.wrap 'x'` on a `u32` wraps at 32 bits even when
stored into a `u64`). **When the values mix sizes, you say which**
(owner, 2026-09-23: b): `('a' plus 'b') plus.wrap 1` with an `i8` and an
`i16` is an error until written `to.i16['a' plus 'b'] plus.wrap 1`, and so is
`'a' plus.wrap 'b'`. Wrapping exists for hashes and counters, where the size
is the algorithm, so it is written and never guessed. Proposed
alternatives: the bigger size; the left operand's (the oracle's old
placeholder).

**A negative exponent is a check site** (owner, 2026-09-24: a), for `xx`
and `xx~` alike: `2 pow -1` is a half, which no integer holds, and a
fraction is not something wrapping answers. the prover sees the whole
program and proves the exponent never negative, the working build
checks where it cannot, and the final build refuses an unproven one.
Proposed alternatives: a fixed answer such as 0, so `xx~` stays
check-free; an exponent that must be unsigned. For `xx~` only the base's
size counts: the exponent is a count, like a shift's, not a value being
wrapped (the oracle's reading, not ruled).

How it was reached: check claude asked which size a group of bare
numbers takes when the name beside it (`i16`) and the place it is
stored (`i64`) disagree. Proposed: (a) the name beside decides; (b)
the stored size decides; (c) split, plain by the stored size, wrapping
by the values; (d) bare numbers exact, a mix of sizes done at the
bigger. The owner first reasoned "the math is done inside `'big'`" (b),
then asked what is best for performance and safety. The answer was
this: every option before it either errors on answers that fit (a, and
b in the middle of a narrower line) or can move a wrap (b). On today's
processors a 16-bit and a 64-bit add cost the same, and the checks are
what cost, and this has the fewest. Owner: "yes, that one". It also
answers mixed sizes (`i16 plus i64`), which nothing had decided.

## A `bin` never holds an infinity or a NaN (owner, 2026-09-29: a)

An operation whose `bin` result would be an infinity or a NaN is a check
site, like integer overflow: `'big' times 10.0` past the largest `f64`,
`1.0 div.near 0.0` and `0.0 div.near 0.0` (division by zero), `-1.0 pow 0.5` (no
number). the prover proves them away where it can, the working build checks,
the final build refuses the rest. Text never makes one either
(2026-09-24). Proposed alternative: IEEE's infinities and NaN, as C has
them.

## `f16` / `f32` math rounds after every step (owner, 2026-09-24: a)

```
let $x: float.bin.32 := 16777216.0;
let $y: float.bin.32 := $x plus 1.0 plus 1.0;      # 16777216: each plus rounds back at f32
```

Each operation on `f16` or `f32` values gives the IEEE result at their
width (nearest-even), as C and the processor do. A bare literal takes the
width of the typed value beside it, and the place a result is stored never
changes it. A result that rounds past the largest is a check site. Anyone
who needs the exact sum chooses a bigger or exact type (`f64`, an
integer, `std::uenr`): "the programmer could've chose a better one". This is
the opposite of plain integer math, which is exact because there 16- and
64-bit cost the same. For bins, 64-bit intermediates would halve SIMD lanes
and could not run on Metal, which has no 64-bit float. Proposed
alternative: 64-bit intermediates rounded once where stored.

## `and` / `or` always work out both sides (owner, 2026-09-25: b)

```
let $ok := check_name($f) and check_age($f);     # both run, left then right
if $i at_most std::ls.len(lend $xs):             # a check that protects the next
    if $xs[pos: $i] eq 5:
        …
    end of if              # step is written as two `if`s
end of if
```

Nothing written on a line is skipped: `and` and `or` evaluate the left
side, then the right, even when the left already decides the answer.
The one-line guard `'i' </== … and 'xs'['i'] == 5` is therefore not a
guard. The read runs when `'i'` is past the end, so it stops the working
build and the final build refuses it. It is safe either way, never
silent. The owner's reason: the code does exactly what it says ("so the
code isn't lying"). Proposed alternatives: stop early, as C, Rust, Python,
Go and JavaScript do, so the one-line guard works and the prover narrows the
right side by the left (recommended by check claude and design claude);
run both and refuse such lines at compile time.

**`and.then` / `or.else` stop once the answer is known** (owner, same
day), for when the second half should depend on the first:

```
if $i at_most std::list.length(lend $xs) and.then $xs[position: $i] equal_to 5:
    …
end of if
if $n equal_to 0 or.else 100 divide.floor $n less_than 2:
    …
end of if
```

A variant of `and`/`or` by the dot rule, like `and.bit`. It reads as the
English: "A, and then B" (only if A held) and "A, or else B" (only if A
failed), the spelling Ada uses (`and then` / `or else`). Same precedence as
`and` / `or`. the prover narrows the right side by the left, so the read and
the division above are proven. Names considered: `and.guard`, `and.early`,
`and.short`. Handling it with `otherwise` was asked about and does not
apply: a list read out of range or a division by zero is a check site,
not a failure. A failing `list.get` was floated for the list case and not
chosen.

## Not yet listed (open)

(Integer `div` (then `/`) and `mod` were decided 2026-09-20: floor division, the
mathematician's `mod`; see the bitwise and wrapping paragraph above.)

## Symbols, only where they read correctly (owner, 2026-10-16)

The owner: "the code with words takes long to skim. So, we might wanna add symbols (like +, -. and stuff),
only if it reads correctly (unlike `%` as mod)", "And also make it good practice to use symbols, and when you
want another variation, you use the word form" (`style.md` 15). Each symbol spells its word's plain form:

| Symbol | Word |
|---|---|
| `+`, `-` between two values | `plus`, `minus` (checked, exact) |
| `x` | `times` (owner: "`x` and `xx`"; the language's spelling until 2026-10-03, then old, now back) |
| `xx` | std's `pow` |
| `<`, `>` | `less_than`, `greater_than` (owner: "Keep < >", and "the long" over `less` / `greater`) |
| a leading `-`: `-5`, `-'x'` | a negative; `negate 'x'` (owner: "`-5`, and `negate 'x'`?") |

A variation is the word: `plus.wrap`, `times.ieee`, `div.floor`, `negate.ieee`. `negate` replaced `neg`
everywhere (`op negate`, `level N { negate }`), `neg` now old. **No symbol, always the word** (each symbol
below an old spelling, refused naming the word): `equal_to`, `not_equal`, `at_most`, `at_least` (`==`, `!=`,
`<=`, `>=`: "do you really read `!=` as not equal?"); `div.…` (`/`), `mod.…` (`%`), which say how they round;
`and` (`&` between values), `and.then`, `or.else` (`&&`, `||`, "Yeah, drop them"); `concat` (`++`); and the
non-operators, `lend` / `lent`, `lend.mut` / `lent.mut` (`&'x'`, `&mut 'x'`, `&@x`), `ptr[T]`, `ptr.mut[T]`,
`ptr.read`, `ptr.write` (`*const T`, `*mut T`, `*'p'`, `change *'p' = v;`) and `exactly` (`*19.99`). `*` and
`**` are old too, their symbols now being `x` and `xx`. `op +`, `op -`, `op x` declare `op plus`, `op minus`,
`op times`; `op xx` is not declared. `^`, `|` and `!` stay free to declare. Until 2026-10-16 (from 2026-10-14)
every one of these symbols spelled its word, and good practice was the words. `examples/symbols.cy`.

## Custom operators — `op` (decided 2026-09-20)

```
rec Vec:
    var x: float.bin.64;
    var y: float.bin.64;
end of rec Vec
op <+> [$a: Vec, $b: Vec] -> Vec:
    ret Vec[x = $a.x plus $b.x, y = $a.y plus $b.y];
end of op <+>
op dot [$a: Vec, $b: Vec] -> float.bin.64:
    ret $a.x times $b.x plus $a.y times $b.y;
end of op dot
op **  [give $a: Vec, $b: float.bin.64] -> Vec:
    … ret give $a;
end of op **

let $w := $u <+> $v;                 # 'u' and 'v' both still yours
let $d := ($u <+> $v) dot $v;       # parenthesise when operators meet
let $s := give $u ** 10;              # this op takes 'a' by value; the caller says so
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
   (`<==`, `>==`, and the compound forms `+=`, `-=`, …) cannot be redefined; `^`, `|`, `!`
   are free to declare, and undeclared they still point at `pow`, `or`,
   `not`. (`*`, `%`, `&` were too, until they became the language's own
   spellings on 2026-10-14, below.)
3. **No precedence** (owner: "Require"). A custom operator next to any
   other operator is a compile error until parenthesised — the `mod`
   rule: where there is no settled reading, parens. Nobody memorises a
   library's table.
4. **Operands are lent by default; `give` on an operand takes it by value,
   and then the caller writes `give` too** (owner: "middle", between
   always-lent and Rust's by-value). Rust's `a plus b` moves both operands,
   which is why every numeric crate writes four `impl`s per operator;
   always-lent reads like arithmetic but cannot reuse a dead operand's
   buffer (`s = s + "x"` in a loop). The middle keeps `'u' <+> 'v'` for
   the common case and lets an `op` say `give 'a'` when it wants the
   memory. the prover may elide the copy in the lent case when the operand is
   dead; the `give` form guarantees it.

Rules that follow: exactly two operands; `--> T` required; an operator
cannot fail (make it a `fn`); it is never called with `[…]`; `not`, `and`,
`or` cannot be redefined; the language's other operators may be **defined
for a user type** (`op plus ['a': Vec, 'b': Vec]`, decided 2026-09-20 with
interfaces) but never changed for the language's own types. Prefix (one-operand) custom
operators are not offered.

### A kit's precedence (owner, 2026-10-03)

```
precedence.low_first:                          # level 1 binds loosest (good practice)
    level 1 [eq, ~=] refused;     # a language operator pins its level
    level 2 [plus] left_first;
    level 3 [times, dot] left_first;
    level 4 [scale] left_first;     # no language operator: above times, by number
end of precedence.low_first
```

A kit may give its operators a precedence in one block; `precedence.high_first`
counts the other way, and a bare `precedence` is refused. Each level is `level N { … }`,
its operators separated by `,`, and after the braces it says how a chain of it reads
(`left_first`, `right_first`, `refused`), which must agree with any language operator
in it. This spelling is the owner's of 2026-10-14 ("That syntax seems good, but the no
commas are bothering me."); it was `level.when_chained.left_first 2 { plus }`, now
refused naming the new one. A level may hold `neg` (a negative), so std's `pow` sits above it. An
operator in no level keeps the custom-operator rule: brackets beside any other.
Operators of two kits in the same gap need brackets between them. The language's
bitwise operators have no precedence and cannot be given one.
`examples/precedence_levels.cy`.

### From another kit (owner, 2026-09-23: a)

**Since 2026-10-02 (owner; built 2026-10-03): a word operator may also carry
its path, `'a' std::pow 'b'`, and a kit may declare narrowed forms after
a dot, `pub op pow.wrap […]`, used `'a' std::pow.wrap 'b'`. Bare and
with a path are both legal; the official kit registry refuses a
published kit that writes another kit's word operator bare, unless the
owner overrides it ("Always refuse, unless I override"). A glyph operator
(`~=`) has no path.** The paragraph below is the 2026-09-23 rule.

A `pub op` is usable by every kit that depends on the one that wrote
it, with nothing written: an operator has no place for a path, so
`'u' dot 'v'` finds the `dot` written for what `'u'` and `'v'` are. Any
kit may share an operator for any types, the language's own included
(`op ~= ['a': f64, 'b': f64] -> bool` for "equal within a margin").
**Two meanings of one operator for the same types is a compile error**
(owner: "shouldn't two meanings of `~=` just err?"), whether they come
from two kits or from a kit and your own code. With no path to pick
one, the fix is to drop a kit, or copy one operator into your own
code and edit it. Proposed alternative (design claude's
recommendation): only the kit that owns one of the operand types may
share an operator, as in Rust, so two can never clash. That would stop
a kit from sharing `~=` or `+|` for the language's own types.

Oracle: `examples/operators.cy`, thirteen reject tests. It runs one
kit, so an operator is found by name across the whole program.
