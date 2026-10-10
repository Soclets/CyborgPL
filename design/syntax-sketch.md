# Syntax sketch

> Revised 2026-10-07: code here is in the new syntax; an old spelling in the prose is history. The old → new table is `revision-2026-10-07.md`.

The language's surface as decided so far. Revised 2026-09-19 (owner):
`fn`, the type after the name, and `[ ]` only where it builds a value;
revised 2026-09-25 (owner): changing says `change`; revised 2026-10-02
(owner): every declaration starts with `let` again (it had none from
2026-09-25).
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
    let 'mine' := Locker[name = "A", number = 7, occupied = false];
    let mut 'total': int32 = 0;
    for 'i' in range[from 1, to 10, by 1] {
        if ('i' mod.floor 2) equal_to 0 { change 'total' = 'total' plus 'i'; }
        else if 'i' equal_to 7 { break; }
        else { continue; }
    }
    let 's' := "line 1" ++ newln ++ "line 2";
    cord::print.useful["total: ", 'total', newln];
}
```

## Declarations (revised 2026-09-19)

```
let 'x' := 1000;                 # Precog picks the best type
let 'x': bin16 = 1000;           # explicit type; must be fully explicit
let mut 'n': int32 = 0;          # mutability, when stated, before the name
let immut 'limit' := 10;
let 'r': cord::uenr = 1000;        # a Limb-provided type: `::` crosses, `.` narrows
pub const 'max' := 8;            # a Limb-level value: top level only, see limbs.md
```

- **`let` declares, `change` changes** (owner, 2026-10-02: "Kinda want
  `let` back (not the entire era), honestly. It just reads more
  naturally"). Every statement that touches a variable starts with a word
  saying what it does: `let 'x' := 1;` declares, `change 'x' = 2;`
  changes, `shadow 'x' := 3;` shadows. History: `var` until 2026-09-19,
  `let` 2026-09-19 to 09-25 (when a change was a bare `'x' = 2;`), no
  word 2026-09-25 to 10-02 (when `change` arrived), `let` again since,
  with `change` kept. `change` is the word because it cannot be read as
  "make a new one".
- **The type follows the name after `:`** — `'x': int32`, never
  `int32 'x'`. The name comes first and long types no
  longer push it off the line. `:` is the one way to attach a type to a
  name: variables, parameters, struct fields, loop counters.
- **`:=` only without a type, `=` only with one.** Unchanged. `:=` marks
  "the compiler chooses the best type"; `=` marks "you said everything".
- **`mut` / `!mut`** (`!mut` is the same as `!mut`) are bare words
  between `let` and the name, and `!$` before those says the name cannot
  be shadowed (2026-09-25): `let !$ mut 'total' := 0;`. Statable, not standard — see `ownership-syntax.md`. **Plain
  means Cortex infers**: `let 'x' := 1; change 'x' = 2;` is legal; `mut` states
  intent, `!mut` is the enforced contract. The same rule holds for an
  owning parameter (`'items': T` may be changed; `!mut 'n': T` may not).
  Not Rust's default-fixed — a fifth cold reader assumed it was
  (2026-09-21).
- Style: a space after `:`, none around `::`.

A `decl` redesign was recorded and dropped on 2026-10-01 (owner: "let's
just go back to the no `let` era"); `let` itself came back the next day,
without the rest of that redesign. See `DECISIONS.md`.

## What the sample establishes

- **Dot chains narrow general→specific inside a type and inside a Neuron
  path** — `str.utf8`, `char.grapheme`, `std::print.useful.stdout`, `lend.mut`.
  A *type parameter* is bracketed, not chained: `list[int32]`,
  `array[list[str.utf8]]` (decided 2026-09-19). They no longer carry the name-to-type job (that is `:`) and no
  longer hold keywords together (`else if`, `while`, `for`). The chains
  that stay are the ones doing real work.
- **Keywords are bare and reserved** (decided 2026-09-14). The language's
  words — `fn`, `change`, `return`, `if`, `else`, `loop`, `while`, `for`, `in`,
  `break`, `continue`, `give`, `copy`, `lend`, `lent`, `op`, `check`,
  `no_check`, `true`, `false`, `mut`, `immut`, `temp`, `perm`, `range`, `seq`,
  `list`, `array`, `struct`, `pub`, `const`, `module`, `neuron`, `my`, `fail`, `otherwise`, `to`, `incl`, `with` (and the retired `var`, `func`; the full list is the oracle's `KEYWORDS`) — cannot
  be used bare as function names; a function *may* be named with one, and
  is then reached only through a path (`std::to[…]`, `my.to[…]`;
  `limbs.md`, 2026-09-19). Variables are quoted and can never collide with
  anything.
- **`::` crosses into a Neuron; `.` narrows within** (decided 2026-09-14).
  `std::print.stored.stdout[…]`, `'r': std::uenr`, `crate_x::Locker[…]`. Inside a
  Neuron the same `.` walks modules: `shapes.circle.area[…]`, `my.helper[…]`
  for the root (`limbs.md`, 2026-09-19; `limb.` until 2026-10-06).
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

  A **single expression stands bare**: `let 'x' := 1000;`, `change 'i' = 'i' plus 1;`,
  `return 'a' plus 'b';`, `if 'a' == 'b' { }`, `let 'n' := add[10, 20];`,
  `occupied = false` inside a construction. A bracket around a single
  expression is allowed as grouping and builds nothing.
- **Text joins with `++`** (decided 2026-09-19, replacing the whitespace
  join of 2026-09-14 — "we'll make it explicit"): `"#" ++ to.stored.str.utf8['n']
  ++ " (" ++ 'owner' ++ ")"`, `"line 1" ++ \n ++ "line 2"` (a bare `\n` is
  a newline text). **Both sides must be text** (later the same day): a
  number is converted first with `to.stored.str.utf8[…]` or
  `to.useful.str.utf8[…]` (`conversion.md`). Two values side by side with nothing between
  (`"a" 'b'`, `[10 20]`) are an error naming `++` and `,`. Precedence: below
  `+ -`, above comparisons, left-associative — `"n=" ++ 'a' + 'b'` joins the
  sum. The 2026-09-14 rule "a bare number is an operand, not a piece" is
  retired with the whitespace join; `"count: " ++ to.stored.str.utf8[10]` is the
  spelling.
- **A string is exactly its characters** (decided 2026-09-19). Inside
  `"…"`, `\` escapes only what cannot otherwise be written in a string:
  `\"` and `\\`. Everything else is literal — `"\n"` is a backslash and an
  `n`, a tab is a typed tab. **The newline is the bare `\n` outside
  quotes**, which is why it lives outside: `"Line 1" ++ \n ++ "Line 2"`,
  `let 'nl' := \n;`. (The oracle's first lexer had C-style `\n`/`\t` inside
  quotes; a default, never decided, now removed.)
- **Names are quoted, so operators are free glyphs**: `let '+' := 10 plus 20;`
  is legal — the name `'+'` and the operator `+` are different tokens.
- **`( )` is arithmetic grouping only**: `'a' plus ('b' times 2)`. Never a call,
  never a parameter list.
- **`print.stored` vs `print.useful`** (decided 2026-09-14; both named since
  2026-10-02, owner: "it has to be required & compatible with `.weird`" —
  a bare `std::print.stdout` is refused, saying which to write):
  `std::print.stored.stdout[…]` prints the real stored value (a `bin32` `0.1` prints as
  `0.100000001490116119384765625`); `std::print.useful.stdout[…]` prints the
  useful value (`0.1`). Both take **any number of values, comma-separated**,
  render each and print them with nothing between (2026-09-19):
  `std::print.useful.stdout["x is ", 'x', \n]` → `x is 0.1`. Alternative
  spelling for `print.useful` (owner, 2026-09-21, after four readers called it
  cryptic): `std::this_will_print_the_useful_numbers.this_will_go_to_the_standard_output[…]`
  (and `…standard_error`). `print.useful` stays. **A number joined
  with `++` becomes text in its stored form, everywhere**:
  `let 's' := "x is " ++ 'x';` holds the full expansion, and `print.useful['s']`
  cannot undo it. `print.useful` is the only thing that produces the useful form,
  and only for the values handed to it directly.
- Borrowed prints (`std::printc`/`std::c.printf`, `std::printrs`/`std::rs.println`)
  existed for a day (2026-09-19) and were removed by the owner on
  2026-09-20; `print` and `print.useful` are the prints.
- **`shadow 'x' := …;` shadows** (owner, 2026-10-08; `$:=` / `$=` from 2026-09-14): redeclaring a visible name
  needs the `$`; `shadow 'x' := 2;`. A name marked `!$` cannot be shadowed; an
  `!mut` one can (2026-09-25: `!mut` only forbids `change`).
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
var 's' := ["line 1" newln "line 2"];
func.int32 add [var.int32 'a', var.int32 'b'] { return ['a' plus 'b']; }
loop.temp.range.int32 'i' [= 0, = 9] { if ['i' equal_to 7] { break; } }
```

The type rode on the keyword chain, every value position wore `[ ]`, and
control flow was chained from `loop`/`else`. The owner revised it on
2026-09-19 for readability: name first, brackets only where they build. The
long form (`variable.IEEE754.Binary:16-bit`) was dropped with it.
