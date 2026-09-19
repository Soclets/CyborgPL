# oracle — the CyborgPL reference interpreter

The executable specification. A tree-walking interpreter in Rust that
implements exactly what `design/` has decided, refuses everything undecided
with a message that says so, and is deliberately slow, small, and obvious.
Its job is to keep the prose honest: if a note in `design/` and this program
disagree, one of them is wrong, and it is usually the note.

Not the compiler. Not Precog. No LLVM. See `design/compiler-verification.md`
for where it fits.

```
cd oracle
cargo build
./target/debug/oracle ../examples/scores.cpls
./target/debug/oracle ../examples/geometry     # a Limb directory: limb.cpls + modules
cargo test          # every examples/*.cpls (or Limb directory) must match its .out;
                    # every tests/reject/*.cpls (or directory) must fail with its .err
```

## What it enforces

Everything in `design/syntax-sketch.md`, `types.md`, `operators.md`,
`functions.md`, `control-flow.md`, `ownership-syntax.md`, and the `MAIN`
entry point — see `tests/reject/` for one program per rule. Highlights:

- quoted names are variables; function names are bare; keywords are reserved
- `let 'x' := v;` / `let 'x': T = v;` — `:=` only without a type, `=` only with one; the typed form is fully explicit; `let mut` / `let immut` before the name (revised 2026-09-19; `var`, `func`, `loop.while`, `else.if`, `loop.temp.for` are refused with the new spelling)
- `fn name ['a': int32, lent 'b': str.utf8, lent.mut 'c': list[int32]] -> int32 { }`; an owning parameter has no keyword; the return type is `-> T` after the parameters
- `[ ]` is written only where it builds: a list (comma separates values), a call's arguments, a struct construction, an index, `range[…]`; a single expression stands bare (`let 'x' := 1000;`, `if 'a' == 'b' { }`); two values side by side are an error
- `++` joins text (2026-09-19): one side must be text, the other is rendered in stored form; `1 ++ 2` is refused; precedence below `+ -`, above comparisons; `print`/`printu` take any number of comma-separated values and render each (stored / useful)
- precedence as mathematics has it, parentheses required where it has no settled answer (`6 / 2 x 3`, `mod` beside anything, chained comparisons, `not` beside a comparison)
- `x xx + - / mod == !== < > <== >== and or not`; `*`, `^`, `%`, `!=`, `>=`,
  `++`, `+=` are refused with a pointer to the right spelling
- no implicit conversion between numeric families; a bare literal adopts its
  context's family; typed widths are range-checked
- `give` / `copy` / `lend` / `lend.mut` at the call, nothing / `lent` /
  `lent.mut` at the parameter, and they must agree; a bare owned variable
  cannot be stored; a given variable is dead until reassigned
- the default check tier: every integer operation is overflow-checked and
  every division zero-checked, because there is no Precog here to remove them
- `check <stmt>` / `nocheck <stmt>` / `check { … }` (2026-09-19): the oracle runs the check anyway; a `check` whose site fires reports the counterexample value, a `nocheck` whose site fires reports the lie; refused on a fn/struct/const/module, inside a value, at the top level, or doubled
- `print` shows the stored value (full binary expansion), `printu` the useful one
- comments: `#`, `#wl`, `#Nl`, `#Nld`
- `MAIN { }` is a block; `std::exit[n]` is `uint8`-checked
- modules (2026-09-19): a Limb directory's `limb.cpls` is the root, every other `.cpls` a module (directories nest), `module name { }` blocks nest; names lowercase ASCII, not device names; bare = this module, dotted = relative, `limb.` = root; visibility nothing / `pub.limb` / `pub` on fn, struct, const, module, checked along the whole path (`pub` and `pub.limb` behave alike inside one Limb); `mod x`, `module x;`, MAIN outside the root, a file-and-block module are refused
- `const 'x' := v;` at the top level only — evaluated before MAIN in order, never assignable/shadowed/verbed, a `std::` call in its value is refused; `let` at the top level is refused
- shadowing only with `$:=` / `$=`; an `immut` name cannot be shadowed
- `struct Name { f: type … }` (bare field names, 2026-09-19), construction by field name `Name[f = v, …]` with every field given, `'s'.f` read / `'s'.f = v;` write, `immut f: type` fields, `give 's'.f` consumes the whole variable; a struct is always owned (verbs required) even with only plain fields
- `list[T]` / `array[T]` types; `'xs'[pos i]` 1-based with range checks, `pos` mandatory; `'xs'[pos i] = v;` element assignment; `let 'xs': list[T] = [];` and `let 'ys' := list.empty;`; `list.length/append/remove[lend…]`, `array.length`; append to an array is an error
- `if c { } else if c { } else { }`, `loop { }`, `while c { }`, `for [seq] [temp|perm] 'x' in lend 'xs' { }`, `for [seq] [perm] 'i'[: T] in range[start, stop, step] { }`: inclusive stop, `temp` is the default, `perm` keeps the last value, the counter/element cannot be assigned, `seq` first then `perm`
- `let immut` cannot be assigned to; `let mut` is accepted and changes nothing
- a `[ ]` inside a `[ ]` is a value; `[1,]` is a one-element list

## Provisional picks — the design has not decided these

Each is marked "provisional" in the source. They are picks the interpreter
had to make to run at all; none is a decision.

| Pick | Where | Why this way |
|---|---|---|
| `/` on integers truncates; `mod` takes the sign of the dividend | `interp.rs` `binary` | Open question |
| A `:=` character literal infers the narrowest unit that holds it (byte → point → grapheme) | `interp.rs` `eval` | Precog would also look at use; the oracle only sees content |
| A `lend` held in a variable cannot be returned from a function or stored in a list | `interp.rs` | The oracle has no lifetimes; the compiler will (Precog infers them) |
| `lend.mut 'x'.field` writes back at the end of the statement; `'x'.field[pos i] = v` is refused (lend the list to a name first) | `interp.rs` `stmt` | A statement is the unit of a lend here; the compiler tracks the borrow itself |
| Grapheme counting is an approximation (combining marks, ZWJ, variation selectors, skin tones) | `interp.rs` `grapheme_count` | The real runtime needs UAX #29 |
| `deci`, `bin128`, `uint128`'s top half, `std::uer` | — | Parsed, refused at run time as not implemented |
| A file module is public; a directory with no `.cpls` of its own is an implicit public module | `main.rs` `load` | A file has no line to carry `pub.limb`; open in `design/limbs.md` |



## Not implemented because not designed

String iteration,
custom operators (`op`),
boundary range annotations, Limb declaration and other Limbs, input, error
handling, concurrency. Each is refused with a message naming it.
