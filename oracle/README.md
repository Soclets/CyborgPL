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
cargo test          # every examples/*.cpls must match its .out;
                    # every tests/reject/*.cpls must fail with its .err
```

## What it enforces

Everything in `design/syntax-sketch.md`, `types.md`, `operators.md`,
`functions.md`, `control-flow.md`, `ownership-syntax.md`, and the `MAIN`
entry point — see `tests/reject/` for one program per rule. Highlights:

- quoted names are variables; function names are bare; keywords are reserved
- `:=` only without a type, `=` only with one; the typed form is fully explicit
- `[ ]` is the value slot; whitespace joins pieces; a comma separates values;
  a bare number is an operand and never a piece
- precedence as mathematics has it, parentheses required where it has no settled answer (`6 / 2 x 3`, `mod` beside anything, chained comparisons, `not` beside a comparison)
- `x xx + - / mod == !== < > <== >== and or not`; `*`, `^`, `%`, `!=`, `>=`,
  `++`, `+=` are refused with a pointer to the right spelling
- no implicit conversion between numeric families; a bare literal adopts its
  context's family; typed widths are range-checked
- `give` / `copy` / `lend` / `lend.mut` at the call, `var` / `lent` /
  `lent.mut` at the parameter, and they must agree; a bare owned variable
  cannot be stored; a given variable is dead until reassigned
- the default check tier: every integer operation is overflow-checked and
  every division zero-checked, because there is no Precog here to remove them
- `print` shows the stored value (full binary expansion), `printu` the useful one
- comments: `#`, `#wl`, `#Nl`, `#Nld`
- `MAIN { }` is a block; `std::exit[n]` is `uint8`-checked
- shadowing only with `$:=` / `$=`; an `immut` name cannot be shadowed
- `list.T` / `array.T` types; `'xs'[pos i]` 1-based with range checks, `pos` mandatory; `'xs'[pos i] = [v];` element assignment; `var.list.T 'xs' = [];` and `var 'ys' := [list.empty];`; `list.length/append/remove[lend…]`, `array.length`; append to an array is an error
- `loop.temp|perm.for` and `loop.temp|perm.range 'i' [:= a, := b, := step]`: inclusive stop, `perm` keeps the last value, the counter/element cannot be assigned
- `var.immut` cannot be assigned to; `var.mut` is accepted and changes nothing
- a `[ ]` inside a `[ ]` is a value; `[1,]` is a one-element list

## Provisional picks — the design has not decided these

Each is marked "provisional" in the source. They are picks the interpreter
had to make to run at all; none is a decision.

| Pick | Where | Why this way |
|---|---|---|
| Whitespace between pieces is a separator only — no implicit space | `interp.rs` `value_mode` | Open question in `DECISIONS.md` |
| `/` on integers truncates; `mod` takes the sign of the dividend | `interp.rs` `binary` | Open question |
| A `:=` character literal infers the narrowest unit that holds it (byte → point → grapheme) | `interp.rs` `eval` | Precog would also look at use; the oracle only sees content |
| A `lend` held in a variable cannot be returned from a function or stored in a list | `interp.rs` | The oracle has no lifetimes; the compiler will (Precog infers them) |
| Grapheme counting is an approximation (combining marks, ZWJ, variation selectors, skin tones) | `interp.rs` `grapheme_count` | The real runtime needs UAX #29 |
| `deci`, `bin128`, `uint128`'s top half, `std::uer` | — | Parsed, refused at run time as not implemented |


## Not implemented because not designed

String iteration, structs,
custom operators (`op`), the check-tier syntax (`check` / `nocheck`),
boundary range annotations, Limb declaration and other Limbs, input, error
handling, concurrency. Each is refused with a message naming it.
