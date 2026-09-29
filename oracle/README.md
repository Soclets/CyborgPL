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

- quoted names are variables; function names are bare; keywords are reserved in bare positions — a keyword-named `fn` is reached only by path (`limb.to[…]`, 2026-09-19)
- `'x' := v;` / `'x': T = v;` — no `let` since 2026-09-25; `:=` only without a type, `=` only with one; `mut` / `immut` / `!mut` before the name, `!$` before those (cannot be shadowed); changing is `change 'x' = v;` (a bare `'x' = v;` and `let` are refused with the new spelling, as are `var`, `func`, `loop.while`, `else.if`, `loop.temp.for`)
- `&'x'` / `&mut 'x'` are accepted wherever `lend 'x'` / `lend.mut 'x'` are (call site, `.field`, `[i]`), and `&'x': T` / `&mut 'x': T` wherever `lent` / `lent.mut` are (parameters, `fn[…]` types) — the build then prints `What are you? Rust programmer or Claude?` on stdout once per `&`, then runs; `std::this_will_print_the_useful_numbers.this_will_go_to_the_standard_output` is `print.useful.stdout` (both 2026-09-21)
- `fn name ['a': int32, lent 'b': str.utf8, lent.mut 'c': list[int32]] -> int32 { }`; an owning parameter has no keyword; the return type is `-> T` after the parameters; `...'rest'` (last only) collects the remaining arguments, each checked as if bound to that parameter (2026-09-19)
- `std::print.format.stdout/stderr[fmt, …]`: C escapes and `%` formatting, checked against the arguments (type, count); `%u`, `%p`, `%n` refused; a non-literal format needs `nocheck` (2026-09-22)
- `concur [.detached] [with […]] { … } [otherwise …]` / `concur f[…]`: tasks with explicit captures (a detached task may not lend); a block waits for the tasks it started; a task's failure lands on its `concur`'s `otherwise` (required when the body can fail); `return`/`break` inside a task refused
- `channel[T]` / `channel[T][0, N]`, `channel.send` (a value is given, never lent), `channel.take … otherwise` (fails `closed` when nothing can ever send), `whichever { 'v' from lend 'c' { } … } otherwise` (fails `closed`); `std::time.after/every` are channels, `std::time.wait` a no-op; duration literals `5 seconds` (nanos…hours, singular/plural) with `== < + - x` (2026-09-22)
- an untyped list still has an element family (what is in it): storing a bin into a list of integers is refused, as into a declared `list[int32]` (hole found by the compiler session, 2026-09-22)
- `list.slice` / `array.slice` with bare positions, 1-based and inclusive, owning the elements; out of range and a backwards span are check sites (2026-09-23)
- `bin` is 16/32/64 — `bin128` is refused by name (2026-09-23); `deci` and `std::uenr` remain unimplemented
- plain integer math (`+ - x xx`) is **exact, and checked where the value lands**: a typed declaration, a `change`, an argument, a return (owner, 2026-09-23). `'v' > ('v' + 'v')` on an `int8` of 112 compares 112 with 224. In between, a value no 64-bit type holds (below `int64`'s minimum or above `uint64`'s maximum) is a check site; bare numbers alone are worked out exactly. This reverses the morning's "checked where it happens" (seed 23), which was a reading, never a ruling
- a shift's, `not.bit`'s and a wrapping operator's width comes from any operand whose type is known — a declared name, a `to.T[…]`, either side of an operator, or the type the value is being stored into; only a bare literal with no context has none (2026-09-23)
- a `for` range's step is its own signed number: `for 'i': uint64 in range[3, 1, -1]` is legal, and `range[3, 0, -1]` stops at 0 without underflowing (2026-09-23, fixed after the verification session found the oracle refusing it)
- `simd[T, N]`: the target's shapes only (the oracle's target is ARM64's 128-bit registers, so `simd[uint8, 16]`, `simd[int32, 4]`, `simd[bin64, 2]`, …); ordinary operators lane by lane, scalar-with-vector broadcasting, comparisons giving `simd[bool, N]` (and so refused as a condition); `simd.fill` (lane count from the declared type) / `load` / `store` (both check the bounds) / `all` / `any` / `max` / `min` / `sum` (2026-09-22)
- `map[K, V][lo, hi]` bounds the entry count (checked on `set`/`remove`; `map.length` is `uint64[lo, hi]`); `set` checks the key and value against K and V (2026-09-22)
- `map["ann" = 5551, …]` literal; `map[]` refused (`map.empty`); a duplicate key in a literal refused (2026-09-22)
- `map.keys/values/pairs[lend 'm']` give lists in insertion order (`pairs` as structs with `key`/`value`); `map.ordered` / `map.unordered` variants, and walking a `map.unordered` is refused (2026-09-22)
- `map[K, V]` with `map.empty/length/has/get/set/remove` (`get`/`remove` fail `missing`, `otherwise` required; `set`/`remove` need `lend.mut`); keys by structure, or by a type's own `op ==`/`fn hash` — and then equal keys hashing differently is a check-site error on the comparison that exposes it; functions, pointers and maps refused as keys (2026-09-22)
- `str.utf8[lo, hi bytes|points|graphemes]` — a length-bounded text; the unit is required; assignment and `++` are checked against it; `to.str.utf8[…bound…][v]` fails `out_of_range` (2026-09-22)
- an `extern fn` with a ranged return (`-> uint64[0, 10]`) can fail `out_of_range` (an `otherwise` is required; `nocheck` does not remove it); a plain-return extern still cannot fail (2026-09-22)
- `extern ['mut'] 'x': T;` — a C global (`extern let` until 2026-09-25), C-shaped, no value; reads and writes are nocheck sites; a read-only one cannot be written; no verbs; emulated as zero until written (2026-09-21)
- `char.code[c]` (point/byte → number; grapheme refused), `char.point.from_code[n]` / `char.byte.from_code[n]` (fail `not_a_char`), `to.char.<unit>[text]` (fails `not_one_char`), `to.char.<unit>[char]` down (fails `not_one_point` / `not_one_byte`) and up (no `otherwise`); `to.int32[char]` and `to.char.point[number]` refused; a char literal takes its declared unit (2026-09-21)
- a module (block or file) and a struct/enum/interface with the same name in one Limb are refused (2026-09-21)
- `to.int32[…]` of a bin fails `not_whole` unless the bin is whole (an `otherwise` is required); `std::round` / `floor` / `ceil` / `trunc` round a bin to a whole bin (2026-09-21)
- `list.remove[lend.mut 'xs', i]` hands the removed element back (a value, or dropped as a statement); `list.append` used as a value is refused; `give 'xs'[i]` stays refused (2026-09-21)
- `...'xs'` in a call unpacks a list into the variadic part; refused when the list's length is not known here (the oracle knows: a literal list, `list.empty`, a bounded type, a bundle — and forgets after `append`/`remove` on an unbounded list), when the function has no variadic, when it would land on a fixed parameter, or outside a call; owned elements into an owning variadic need `...copy` / `...give` (give empties the list), a verb on plain elements or into a `lent ...` is refused (2026-09-21)
- `list[T][lo, hi]` is a length-bounded list: construction, assignment, `append` and `remove` are checked against the bound; its `list.length` is `uint64[lo, hi]`; `array[T][…]` is refused (2026-09-21)
- `list.length` / `array.length` are `uint64`: an untyped `:=` from one takes that type, so a later negative assignment is refused (2026-09-21)
- `[ ]` is written only where it builds: a list (comma separates values), a call's arguments, a struct construction, an index, `range[…]`; a single expression stands bare (`'x' := 1000;`, `if 'a' == 'b' { }`); two values side by side are an error
- `++` joins text (2026-09-19): both sides must be text (`to.str.utf8[…]` converts); `to.<type>[value]` converts — text→number fails with `not_a_number` and takes `otherwise`, narrowing is a check site, bin→int / bool / char / list / struct are refused; precedence below `+ -`, above comparisons; `print`/`print.useful` take any number of comma-separated values and render each (stored / useful)
- precedence as mathematics has it, parentheses required where it has no settled answer (`6 div 2 x 3`, `mod` beside anything, chained comparisons, `not` beside a comparison)
- `x xx + - / mod ++ == !== < > </== >/== and or not`, wrapping `+~ -~ x~ xx~` (the oracle wraps at the left operand's declared width — a typed variable — and refuses otherwise); bitwise `and.bit or.bit xor not.bit shl shr` (parenthesise beside any operator; `not.bit` and `shl` overflow need a declared width); `!=`, `>=`, `<=`, `--`, `+=`, `-=` are refused with a pointer to the right spelling; `*`, `^`, `%`, `&`, `|`, `!` are refused unless declared with `op`
- no implicit conversion between numeric families; a bare literal adopts its
  context's family; typed widths are range-checked
- `give` / `copy` / `lend` / `lend.mut` at the call, nothing / `lent` /
  `lent.mut` at the parameter, and they must agree; a bare owned variable
  cannot be stored; a given variable is dead until reassigned
- the default check tier: every landing and every 64-bit limit is checked, and
  every division zero-checked, because there is no Precog here to remove them
- `check <stmt>` / `nocheck <stmt>` / `check { … }` (2026-09-19): the oracle runs the check anyway; a `check` whose site fires reports the counterexample value, a `nocheck` whose site fires reports the lie; refused on a fn/struct/const/module, inside a value, at the top level, or doubled
- `print` shows the stored value (full binary expansion), `print.useful` the useful one (the borrowed C/Rust prints of 2026-09-19 were removed 2026-09-20 and are refused by name)
- strings are exactly their characters; only `\"` and `\\` escape; `"\n"` is two characters, the newline is the bare `\n` (2026-09-19)
- comments: `#`, `#wl`, `#Nl`, `#Nld`
- `MAIN { }` is a block; `std::exit[n]` is `uint8`-checked
- modules (2026-09-19): a Limb directory's `limb.cpls` is the root, every other `.cpls` a module (directories nest), `module name { }` blocks nest; names lowercase ASCII, not device names; bare = this module, dotted = relative, `limb.` = root; visibility nothing / `pub.limb` / `pub` on fn, struct, const, module, checked along the whole path (`pub` and `pub.limb` behave alike inside one Limb); `mod x`, `module x;`, MAIN outside the root, a file-and-block module are refused
- failures (2026-09-19): `-> T or fail[a, b: type]`, `fail a;` / `fail b[v];`, `call[…] otherwise value | return … | fail | { a { } b 'v' { } }`; the caller's side is checked before the call runs; `examples/failures.cpls`
- FFI (design/ffi.md, 2026-09-20): `extern fn …;` (C-shaped types only, no `lent`, no `fail`, no body), a call is refused outside `nocheck`; `ptr[T]`/`ptr.mut[T]` only in extern signatures and `layout.c` fields, a `lend`/`lend.mut` becomes the pointer; `ptr.read/write/at` are nocheck sites (past-the-end or null under nocheck = "lied"), `ptr.is_null` is not; `layout.c struct` fields must be C-shaped; `size.of`/`align.of` are C's sizes; `nocheck inline.ASM { raw } with ['v' -> x0, 'n' <- x0, clobber x16];` is parsed and checked, then refused to run; emulated libc: write, exit, getenv, strlen, malloc, free, qsort (calls a Cyborg comparator); a callback must be capture-free and C-shaped (2026-09-21); `ptr[T]` may appear in any fn signature, not in a variable declaration
- text (design/strings.md, 2026-09-20): `str.bytes/points/graphemes[lend 's']`, `str.each.<unit>[…]`; `str.length`, `'s'[…]`, `<` on text, and a bare `for` over a text are refused; emulated std: `std::str.split/join/contains/starts_with/ends_with/trim/upper/lower/replace`, `at.<unit>` and `slice.grapheme` (fail out_of_range), `find.grapheme` (fail missing)
- enums (design/enums.md, 2026-09-20): `enum S { a  b: int32 }`, `S.a`, `S.b[1]`, `match 's' { a { } b 'v' { } }` with every case covered; an enum is owned; `==` works; `match` on a non-enum is refused
- `div` (2026-09-29; `/` before) on integers floors and `mod` has the divisor's sign (2026-09-20); `std::div.trunc` / `std::mod.trunc` truncate
- custom operators (design/operators.md, 2026-09-20): `op <+> ['a': V, 'b': V] -> V { }`, `op dot […]`, `op ** [give 'a': V, 'b': bin64] -> V { }`; a run of `! % & * + - / < > = ? @ ^ | ~` is one operator; no precedence (parenthesise); operands lent unless `give`; the language's operators and near-misses cannot be redefined; an op is never called with `[…]`
- generics (design/generics.md, 2026-09-20): `<T>` in fn signatures, struct fields, enum cases; `Pair[int32, str.utf8]` fills blanks by first appearance; a `<T>` binds to the first value's kind and every later one must match; `size.of[<T>]` and `<T>` in an extern are refused
- interfaces (design/interfaces.md, 2026-09-20): `interface I { fn …;  op …; }`, `<T: I>` bounds checked when a call fills the blank, `V satisfies I;` claims checked at load; the language's operators may be defined for user types (`op < ['a': V, 'b': V]`), picked by the left operand's type; verbs on a `<T>` value are allowed and do nothing for a plain type
- input and range types (design/input.md, 2026-09-20): `std::input.line[]` / `.all[]` read stdin, fail `closed`; `int32[0, 100]` / `bin64[0.0, 1.0]` types checked on every store (a site), `to.int32[0, 100][…]` fails `out_of_range`; bounds must match the family and be ordered; ranges cannot cross to C
- borrows (2026-09-29, check claude's findings): a borrow used after what it borrows was changed, given away or lent mutably is refused, and so is changing a list while a `for … in lend` loop goes through it; a `-> lent@x` result lives while everything in @x does; `return lend 'x'` is checked where it is written, called or not
- `op +` / `op neg` / `op dot`: an operator as a value (2026-09-29); `op and.then` / `op or.else` refused
- functions as values (design/functions-as-values.md, 2026-09-21): a bare name is the function, `'f'[args]` calls it (a list read looks the same since `pos` went, 2026-09-27; what the variable holds decides), `fn[lent T] -> R` types checked by shape, `fn […] -> T with [lend 'v'] { }` captures explicitly (a silent capture is "not declared"); `lend`/`copy 'xs'[i]`
- `const 'x' := v;` at the top level only — evaluated before MAIN in order, never assignable/shadowed/verbed, a `std::` call in its value is refused; a variable at the top level is refused
- shadowing only with `$:=` / `$=`; an `immut` name cannot be shadowed
- `struct Name { f: type … }` (bare field names, 2026-09-19), construction by field name `Name[f = v, …]` with every field given, `'s'.f` read / `'s'.f = v;` write, `immut f: type` fields, `give 's'.f` consumes the whole variable; a struct is always owned (verbs required) even with only plain fields
- `list[T]` / `array[T]` types; `'xs'[i]` 1-based with range checks, no `pos` since 2026-09-27; `change 'xs'[i] = v;` element change; `'xs': list[T] = [];` and `'ys' := list.empty;`; `list.length/append/remove[lend…]`, `array.length`; append to an array is an error
- `if c { } else if c { } else { }`, `loop { }`, `while c { }`, `for [seq] [temp|perm] 'x' in lend 'xs' { }`, `for [seq] [perm] 'i'[: T] in range[start, stop, step] { }`: inclusive stop, `temp` is the default, `perm` keeps the last value, the counter/element cannot be assigned, `seq` first then `perm`
- `immut` (or `!mut`) cannot be changed; `mut` is accepted and changes nothing; `!$` cannot be shadowed, and neither can a loop's counter
- a `[ ]` inside a `[ ]` is a value; `[1,]` is a one-element list

## Provisional picks — the design has not decided these

- **SIMD**: the oracle's target is fixed as ARM64 128-bit (the real one comes from `Synthia.cplm`); a bare `simd.fill[0]` with no declared type is 16 lanes; `simd.sum` overflow is a check site but `max`/`min` are not; broadcasting a scalar against a vector (`'v' + 1`) is allowed, which `design/simd.md` does not mention either way.

- **The oracle's task schedule**: a task runs when something waits on it (a `take`, a `whichever`) or when its block ends, in source order; a sender never waits (the oracle cannot suspend a task mid-body, so a hand-off or a full bounded channel just queues — the waiting is unobservable in output); time never passes unless nothing else can run, and then a timer channel fires at once. One valid schedule of the design's, chosen for determinism, not the only one.


- The oracle is not parameterised by target (pointer size, `bin128` in hardware or not); it behaves as one 64-bit machine. Cross-target proofs would need it to take those as inputs (noted 2026-09-21).


Each is marked "provisional" in the source. They are picks the interpreter
had to make to run at all; none is a decision.

| Pick | Where | Why this way |
|---|---|---|
| A `:=` character literal infers the narrowest unit that holds it (byte → point → grapheme) | `interp.rs` `eval` | Precog would also look at use; the oracle only sees content |
| A `lend` held in a variable cannot be returned from a function or stored in a list | `interp.rs` | The oracle has no lifetimes; the compiler will (Precog infers them). Confirmed a gap, not the design, 2026-09-21 |
| `lend.mut 'x'.field` writes back at the end of the statement; `'x'.field[i] = v` is refused (lend the list to a name first) | `interp.rs` `stmt` | A statement is the unit of a lend here; the compiler tracks the borrow itself |
| Grapheme counting is an approximation (combining marks, ZWJ, variation selectors, skin tones) | `interp.rs` `grapheme_count` | The real runtime needs UAX #29 |
| `deci`, `bin128`, `uint128`'s top half, `std::uenr` | — | Parsed, refused at run time as not implemented |
| A pointer into a lent `str` reads its UTF-8 bytes and yields 0 one past the end; a normal struct's `size.of` uses C's rule | `interp.rs` `ptr_op`, `size_align` | The oracle has no memory; Precog's real layout is not modelled |
| A file module is public; a directory with no `.cpls` of its own is an implicit public module | `main.rs` `load` | A file has no line to carry `pub.limb`; open in `design/limbs.md` |
| `shl`/`shr`/`not.bit` on an exact plain sum of mixed sizes take the left operand's declared type | `interp.rs` bitwise branch | Wrapping was ruled (write the size, 2026-09-23); shifts were not asked |
| `xx~` wraps at the base's size, and its exponent's size does not count as mixing | `interp.rs` wrapping branch | By analogy with a shift's count; the owner ruled mixing for values, not for a power's exponent |
| A bin operation on two different widths (`bin32` with `bin64`) rounds at the wider | `interp.rs` `bin_width` | Mixed bin widths were not asked; the owner ruled per-step rounding at the values' width |
| A C enum or ranged value coming from C (a return, a lent `ptr.mut`, `ptr.read`, a C variable) is checked and fails `out_of_range` — not exercised, since no emulated libc function hands one over; `to.Key[…]` and the callback rule are | `interp.rs` | The rules are decided; the oracle runs no real C |
| `#strict`'s proof rule (every check site proven from its own function) is not modelled; its spelling rules are | `interp.rs`, `parser.rs` | The oracle has no proofs; Precog enforces the proof rule |
| `#strict` goes once at the top of a file, before any item, and covers the file and its `module` blocks | `parser.rs` `items` | Placement and reach were not asked |
| Text → bin also takes a leading `+`, and refuses `_` grouping, `.5`, `5.` and surrounding spaces | `interp.rs` `bin_text` | The owner ruled sign, digits, fraction and exponent; these edges were not asked |
| A list/array mismatch is caught where the source's type is declared (a variable, a call's return, a slice, a conversion); a field read is not traced | `interp.rs` `coll_kind` | The oracle's limit, not the rule |
| `return lend 'x'` is checked without a call; a borrow returned from a call or another lend is checked when the function runs, and a chain of calls is followed 8 deep | `interp.rs` `check_returned_borrows` / `call_deps` | The oracle's limit |
| A borrow, a change of what it borrows, then a use of the borrow is refused on every path when read top to bottom (`lend 's'` or a `-> lent@x` call); without `@x` a call's result borrows whatever its body may hand back on any path; a loop's second trip round is caught only on the path the run takes; a stale borrow is refused where it is next used; a change to a field borrowed with `lend 'x'.f`, and a `for` over an element's lend, are not traced | `interp.rs` `wrote` / `stale` | The oracle's limit: Precog refuses on any path, at compile time |
| `'f' := op +;` with no type is refused | `parser.rs` `decl` | Nothing says which `+`; design claude's reading (2026-09-29) |
| A custom operator as a value (`op dot`) has its operands' kinds, lent unless `give`; with several `op` of one name the oracle refuses rather than picking by where it goes | `interp.rs` `OpValue` | CCA picks by type; the oracle has no expected type at that point |

## Not implemented because not designed

String iteration,
custom operators (`op`),
boundary range annotations, Limb declaration and other Limbs, input,
concurrency, a general variant type. Each is refused with a message naming it.
