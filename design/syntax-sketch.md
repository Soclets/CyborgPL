# Syntax sketch

Owner's first concrete direction, 2026-09-14. Not yet fully decided — see
the open questions at the bottom and in `DECISIONS.md`.

## The sample

Long form (every level of the chain spelled out):

```
variable.IEEE754.Binary:16-bit 'x' = 1000;
variable 's' := ["line 1" \n "line 2"];
standard.print.standardoutput['s' 'x' \n];
```

Short form — the owner's preference, and Claude's recommendation:

```
var.bin16 'x' = 1000;
var 's' := ["line 1" \n "line 2"];
std.print.stdout['s' 'x' \n];
```

## What the sample establishes

- **Dot chains are paths from general to specific**, not method calls.
  `var.bin16` narrows "variable" to "16-bit binary-float variable";
  `std.print.stdout` narrows the standard library down to one action.
- **`:=` appears only in the untyped form.** With a type, plain `=`. The
  glyph marks "the compiler chooses the best type".
- **User names are quoted** (`'x'`, `'s'`); keywords are bare paths. So there
  are no reserved words and names may contain anything.
- **`bin16` is IEEE 754 binary16** — a half-precision float, not a 16-bit
  integer.
- **`[ ]` is the value bracket — "one value, built from what's inside" —
  and it is mandatory in every value position** (decided 2026-09-14). Text
  pieces, whitespace-separated with no commas, join into one text; a bare
  `\n` is a newline piece, so `'s'` above is the single string
  `line 1⏎line 2`. A formula evaluates: `[10 + 20 + 30]`. A lone literal is
  still bracketed: `var.bin16 'x' = [1000];`. `std.print.stdout[...]` takes
  the same construct.
- **Inside `[ ]`, a bare number is an operand, not a piece** (decided
  2026-09-14). Pieces — `"text"`, `'variable'`, `\n` — join as text. A bare
  number must be part of an operator expression; `[10 20]` is a compile
  error.
- **Names are quoted, so operators are free glyphs**: `var '+' := [10 + 20];`
  is legal — the name `'+'` and the operator `+` are different tokens.
- Statements end with `;`.

## Open

- Whitespace between pieces in `[ ]`: separator only (pieces concatenate
  directly) or an implicit space?
- `["count: " 10]`: error, or implicit conversion?
- Mixed precedence: parentheses required, or standard precedence?
- Operator glyphs and explicit conversion spelling.
- With `[ ]` taken for text, how is a list written?
- The rest of the numeric family: integer type names alongside `bin16/32/64`;
  whether `bin128` exists.
- Whether the long form is also valid, or only the short form is the language.
