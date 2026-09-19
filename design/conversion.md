# Conversion — `to.<type>[…]`

Decided 2026-09-19. Nothing converts by itself; a conversion is spelled as
`to`, the target type, and the value in brackets.

```
to.str.utf8['avg']                 # number → text, in its stored form; never fails
to.bin64['n']                      # int → bin, widening; never fails
to.int8['big']                     # narrowing: a check site (does it fit?), like a typed assignment
to.int32["42"] otherwise 0         # text → number: can fail with `not_a_number`
to.int32["7"] otherwise { not_a_number { std::exit[2]; } }
```

- **`to`** is a keyword and the head of the chain, like `list` in
  `list.length[…]`; the chain narrows to the target type (`to.str.utf8`,
  never a bare `to.str`); the bracket holds exactly one value. Owner's
  spelling, over `int32[…]` (types-as-calls) and `std::to.int32[…]` (a
  library job): the compiler owns conversion — Precog has to know exactly
  what int→bin does to prove ranges — and a bare head reads as what it is.
- **A conversion that can fail fails the normal way**: text → number is
  `-> … or fail[not_a_number]` and takes `otherwise` like any call
  (`failures.md`). A narrowing conversion is not a failure but a **check
  site**: `to.int8['big']` carries a runtime check unless Precog proves the
  fit, and `check to.int8[…]` asks for the proof.
- **`++` takes text on both sides.** This closes the language's last
  implicit conversion (two cold readers flagged it): `"n=" ++ 'n'` is an
  error pointing at `to.str.utf8`. `print`/`printu` still render any value
  handed to them directly — rendering for output is what they are for, not
  a conversion.
- A number becomes text in its **stored form**, always (2026-09-14). A
  useful-form conversion (`0.1` rather than the 55 digits) has no spelling
  yet — `printu` is the only thing that produces it.
- Not decided, refused by the oracle: bin → int (round, truncate, or refuse
  a fraction?); anything → `bool` (no truthiness — compare instead); to or
  from `char`, lists, structs.
