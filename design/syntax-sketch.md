# Syntax sketch

The language's surface as decided so far. Revised 2026-09-19 (owner):
`fn`, the type after the name, and `[ ]` only where it builds a value;
revised 2026-09-25 (owner): no `let` — a declaration starts with its name,
and changing says `change`.
The 2026-09-14 form is under **History** at the bottom.

## The sample

```
struct Locker {
    name: str.utf8
    immut number: int32
    occupied: bool
}

fn count [lent 'text': str.utf8] -> int32 { … }

MAIN {
    'mine' := Locker[name = "A", number = 7, occupied = false];
    mut 'total': int32 = 0;
    for 'i' in range[1, 10] {
        if ('i' mod 2) == 0 { change 'total' = 'total' + 'i'; }
        else if 'i' == 7 { break; }
        else { continue; }
    }
    's' := "line 1" ++ \n ++ "line 2";
    std::printu.stdout["total: ", 'total', \n];
}
```

## Declarations (revised 2026-09-19)

```
'x' := 1000;                 # Precog picks the best type
'x': bin16 = 1000;           # explicit type; must be fully explicit
mut 'n': int32 = 0;          # mutability, when stated, before the name
immut 'limit' := 10;
'r': std::uenr = 1000;        # a Limb-provided type: `::` crosses, `.` narrows
pub const 'max' := 8;            # a Limb-level value: top level only, see limbs.md
```

- **No declaration keyword** (2026-09-25; it was `let` from 2026-09-19,
  `var` before). The quotes already say "this is a variable", and what
  follows the name says what happens to it: `'x' := 1;` declares,
  `change 'x' = 2;` changes, `'x' $:= 3;` shadows. `change` is the word
  because it cannot be read as "make a new one".
- **The type follows the name after `:`** — `'x': int32`, never
  `int32 'x'`. The name comes first and long types no
  longer push it off the line. `:` is the one way to attach a type to a
  name: variables, parameters, struct fields, loop counters.
- **`:=` only without a type, `=` only with one.** Unchanged. `:=` marks
  "the compiler chooses the best type"; `=` marks "you said everything".
- **`mut` / `immut`** (`!mut` is the same as `immut`) are bare words before
  the name, and `!$` before those says the name cannot be shadowed
  (2026-09-25). Statable, not standard — see `ownership-syntax.md`. **Plain
  means Precog infers**: `'x' := 1; change 'x' = 2;` is legal; `mut` states
  intent, `immut` is the enforced contract. The same rule holds for an
  owning parameter (`'items': T` may be changed; `immut 'n': T` may not).
  Not Rust's default-fixed — a fifth cold reader assumed it was
  (2026-09-21).
- Style: a space after `:`, none around `::`.

## What the sample establishes

- **Dot chains narrow general→specific inside a type and inside a Limb
  path** — `str.utf8`, `char.grapheme`, `std::printu.stdout`, `lend.mut`.
  A *type parameter* is bracketed, not chained: `list[int32]`,
  `array[list[str.utf8]]` (decided 2026-09-19). They no longer carry the name-to-type job (that is `:`) and no
  longer hold keywords together (`else if`, `while`, `for`). The chains
  that stay are the ones doing real work.
- **Keywords are bare and reserved** (decided 2026-09-14). The language's
  words — `fn`, `change`, `return`, `if`, `else`, `loop`, `while`, `for`, `in`,
  `break`, `continue`, `give`, `copy`, `lend`, `lent`, `op`, `check`,
  `nocheck`, `true`, `false`, `mut`, `immut`, `temp`, `perm`, `range`, `seq`,
  `list`, `array`, `struct`, `pub`, `const`, `module`, `limb`, `check`, `nocheck`, `fail`, `otherwise`, `to`, `with` (and the retired `var`, `func`) — cannot
  be used bare as function names; a function *may* be named with one, and
  is then reached only through a path (`std::to[…]`, `limb.to[…]`;
  `limbs.md`, 2026-09-19). Variables are quoted and can never collide with
  anything.
- **`::` crosses into a Limb; `.` narrows within** (decided 2026-09-14).
  `std::print.stdout[…]`, `'r': std::uenr`, `crate_x::Locker[…]`. Inside a
  Limb the same `.` walks modules: `shapes.circle.area[…]`, `limb.helper[…]`
  for the root (`limbs.md`, 2026-09-19).
- **User names are quoted** (`'x'`, `'s'`). **Settled and reaffirmed — see
  `DECISIONS.md`; not open for relitigation.** The word operators (`x`,
  `mod`, `and`, …) and the bare keywords depend on it.
- **`bin16` is IEEE 754 binary16** — a half-precision float.
- **`[ ]` is written only where it builds a value** (decided 2026-09-19,
  replacing "mandatory in every value position"). It builds when it
  - collects a list: `[1, 2, 3]`, `[7,]` for one element, `[]` empty where a
    type says list;
  - carries a call's arguments: `add[10, 20]`, `std::exit[1]`;
  - constructs a struct: `Locker[name = "A", …]`;
  - indexes: `'xs'[3]`;
  - bounds a range: `range[1, 10, 2]`.

  A **single expression stands bare**: `'x' := 1000;`, `change 'i' = 'i' + 1;`,
  `return 'a' + 'b';`, `if 'a' == 'b' { }`, `'n' := add[10, 20];`,
  `occupied = false` inside a construction. A bracket around a single
  expression is allowed as grouping and builds nothing.
- **Text joins with `++`** (decided 2026-09-19, replacing the whitespace
  join of 2026-09-14 — "we'll make it explicit"): `"#" ++ to.str.utf8['n']
  ++ " (" ++ 'owner' ++ ")"`, `"line 1" ++ \n ++ "line 2"` (a bare `\n` is
  a newline text). **Both sides must be text** (later the same day): a
  number is converted first with `to.str.utf8[…]` (`conversion.md`), and
  becomes text in its stored form. Two values side by side with nothing between
  (`"a" 'b'`, `[10 20]`) are an error naming `++` and `,`. Precedence: below
  `+ -`, above comparisons, left-associative — `"n=" ++ 'a' + 'b'` joins the
  sum. The 2026-09-14 rule "a bare number is an operand, not a piece" is
  retired with the whitespace join; `"count: " ++ to.str.utf8[10]` is the
  spelling.
- **A string is exactly its characters** (decided 2026-09-19). Inside
  `"…"`, `\` escapes only what cannot otherwise be written in a string:
  `\"` and `\\`. Everything else is literal — `"\n"` is a backslash and an
  `n`, a tab is a typed tab. **The newline is the bare `\n` outside
  quotes**, which is why it lives outside: `"Line 1" ++ \n ++ "Line 2"`,
  `'nl' := \n;`. (The oracle's first lexer had C-style `\n`/`\t` inside
  quotes; a default, never decided, now removed.)
- **Names are quoted, so operators are free glyphs**: `'+' := 10 + 20;`
  is legal — the name `'+'` and the operator `+` are different tokens.
- **`( )` is arithmetic grouping only**: `'a' + ('b' x 2)`. Never a call,
  never a parameter list.
- **`print` vs `printu`** (decided 2026-09-14): `std::print.stdout[…]`
  prints the real stored value (a `bin32` `0.1` prints as
  `0.100000001490116119384765625`); `std::printu.stdout[…]` prints the
  useful value (`0.1`). Both take **any number of values, comma-separated**,
  render each and print them with nothing between (2026-09-19):
  `std::printu.stdout["x is ", 'x', \n]` → `x is 0.1`. Alternative
  spelling for `printu` (owner, 2026-09-21, after four readers called it
  cryptic): `std::this_will_print_the_useful_numbers.this_will_go_to_the_standard_output[…]`
  (and `…standard_error`). `printu` stays. **A number joined
  with `++` becomes text in its stored form, everywhere**:
  `'s' := "x is " ++ 'x';` holds the full expansion, and `printu['s']`
  cannot undo it. `printu` is the only thing that produces the useful form,
  and only for the values handed to it directly.
- Borrowed prints (`std::printc`/`std::c.printf`, `std::printrs`/`std::rs.println`)
  existed for a day (2026-09-19) and were removed by the owner on
  2026-09-20; `print` and `printu` are the prints.
- **`$:=` / `$=` shadow** (decided 2026-09-14): redeclaring a visible name
  needs the `$`; `'x' $:= 2;`. `immut` names cannot be shadowed.
- **Numbers** (decided 2026-09-17): digit grouping is `_` only
  (`1_000_000`); `1,000` is an error that points at `_`; leading zeros are
  illegal.
- **Comments** (decided 2026-09-17), all beginning with `#`: `#` to end of
  line; `#wl` the whole line including code before it; `#Nl` N lines
  counting this one; `#Nld` this line and N more down. `#x` is an error.
- Statements end with `;`. Bodies are `{ }`.

## Open


Closed 2026-09-14: the typed form is always fully explicit (no bare
`int`/`str`/`char`); character literals are backticked.

## History

2026-09-14 → 2026-09-18 the same sample read:

```
var.bin16 'x' = [1000];
var 's' := ["line 1" \n "line 2"];
func.int32 add [var.int32 'a', var.int32 'b'] { return ['a' + 'b']; }
loop.temp.range.int32 'i' [= 0, = 9] { if ['i' == 7] { break; } }
```

The type rode on the keyword chain, every value position wore `[ ]`, and
control flow was chained from `loop`/`else`. The owner revised it on
2026-09-19 for readability: name first, brackets only where they build. The
long form (`variable.IEEE754.Binary:16-bit`) was dropped with it.
