# Syntax sketch

Owner's first concrete direction, 2026-09-14. Not yet fully decided — see
the open questions at the bottom and in `DECISIONS.md`.

## The sample

Long form (every level of the chain spelled out):

```
variable.IEEE754.Binary:16-bit 'x' = 1000;
variable 's' := ["line 1" \n "line 2"];
standard::print.standardoutput['s' 'x' \n];
```

Short form — the owner's preference, and Claude's recommendation:

```
var.bin16 'x' = 1000;
var 's' := ["line 1" \n "line 2"];
std::print.stdout['s' 'x' \n];
```

## What the sample establishes

- **Dot chains are paths from general to specific**, not method calls.
  `var.bin16` narrows "variable" to "16-bit binary-float variable";
  `std::print.stdout` narrows the standard library down to one action.
- **Keywords are bare and reserved** (decided 2026-09-14, replacing the
  earlier `lang:` prefix). The language's words — currently `func`, `var`,
  `return`, `if`, `else`, `loop`, `in`, `break`, `continue`, `give`, `copy`,
  `lend`, `lent`, `op`, `check`, `nocheck`, `true`, `false`, `mut`, `immut` — cannot be function names. Every
  other language does this; the `lang:` prefix bought "zero reserved words"
  at five characters per statement, and the owner judged nobody would want
  a function named `return` anyway. Variables remain quoted and can never
  collide with anything. Three origins are still visible: a keyword is a
  reserved bare word, `std::…` is a Limb, `'…'` is a variable, and a bare
  non-keyword before `[` is a call in this Limb.
- **`::` crosses into a Limb; `.` narrows within** (decided 2026-09-14).
  Anything that comes from a Limb is reached with `::` after the Limb's
  name: `std::print.stdout[...]`, `var.std::uer 'n' = [1000];`. The chain
  continues across the boundary, so a Limb-provided type sits in a `var`
  chain exactly like a built-in one. (The sample above originally read
  `std.print.stdout`; updated.)
- **`:=` appears only in the untyped form.** With a type, plain `=`. The
  glyph marks "the compiler chooses the best type".
- **User names are quoted** (`'x'`, `'s'`); keywords are bare paths. So there
  are no reserved words and names may contain anything. **Settled and
  reaffirmed — see `DECISIONS.md`; not open for relitigation.** The word
  operators (`x`, `mod`, `and`, …) depend on it.
- **`bin16` is IEEE 754 binary16** — a half-precision float, not a 16-bit
  integer.
- **`[ ]` is the value bracket — "one value, built from what's inside" —
  and it is mandatory in every value position** (decided 2026-09-14). Text
  pieces, whitespace-separated with no commas, join into one text; a bare
  `\n` is a newline piece, so `'s'` above is the single string
  `line 1⏎line 2`. A formula evaluates: `[10 + 20 + 30]`. A lone literal is
  still bracketed: `var.bin16 'x' = [1000];`. `std::print.stdout[...]` takes
  the same construct.
- **Inside `[ ]`, a bare number is an operand, not a piece** (decided
  2026-09-14). Pieces — `"text"`, `'variable'`, `\n` — join as text. A bare
  number must be part of an operator expression; `[10 20]` and
  `["count: " 10]` are both compile errors. A number that needs to appear in
  text has a name first: `["count: " 'n']`.
- **Names are quoted, so operators are free glyphs**: `var '+' := [10 + 20];`
  is legal — the name `'+'` and the operator `+` are different tokens.
- **`print` vs `printu`** (decided 2026-09-14): `std::print.stdout[...]`
  prints the real stored value (a `bin32` `0.1` prints as
  `0.100000001490116119384765625`); `std::printu.stdout[...]` prints the
  useful value (`0.1`). Identical for every type except `bin`.
  **A number piece becomes text in its stored form, everywhere** (decided
  2026-09-14): `var 's' := ["x is " 'x'];` holds the full expansion, and
  `print['s']` shows what `'s'` stores. `printu` is the only thing that
  produces the useful form, and only for its own argument — including its
  own pieces: `printu["x is " 'x']` prints `x is 0.1`.
- **`$:=` / `$=` shadow** (decided 2026-09-14): redeclaring a visible name
  needs the `$`; without it, an error. `immut` names cannot be shadowed.
- Statements end with `;`.

## Open

- Whitespace between pieces in `[ ]`: separator only (pieces concatenate
  directly) or an implicit space?
- Mixed precedence: parentheses required, or standard precedence?
- Explicit conversion spelling.
- Whether the long form is also valid, or only the short form is the language.

Closed 2026-09-14: the typed form is always fully explicit (no bare
`var.int`/`var.str`/`var.char`); character literals are backticked.
