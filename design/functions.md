# Functions

> Revised 2026-10-07: code here is in the new syntax; an old spelling in the prose is history. The old → new table is `revision-2026-10-07.md`.

Decided 2026-09-14; spelling revised 2026-09-19 (`fn`, name-then-type
parameters, `--> T`).

```
fn add ['a': int32, 'b': int32] -> int32 {
    return 'a' plus 'b';
}
let 'sum' := add[10, 20];              # this Limb: bare
let 'big' := mathstuff::pow[2, 64];    # another Limb: Limb::path
```

## Definition

```
fn count [lent 'text': str.utf8] -> int32 { … }     # looks at it
fn edit  [lent.mut 'text': str.utf8] { … }          # may modify it
fn keep  ['text': str.utf8] { … }                   # owns it
fn twice [immut 'n': int32] -> int32 { … }          # stated mutability
```

- **`fn`**, bare name, parameters in one `[ ]`, optional `--> type`, body in
  `{ }`. `pub fn` makes it visible to other Neurons (`limbs.md`).
- **Parameters are `'name': type`.** The ownership verb, when there is one,
  sits before the name and mirrors the caller exactly: the caller writes
  `lend 's'`, the parameter reads `lent 'text': …`; `lend.mut` ↔ `lent.mut`.
  An **owning parameter has no keyword** — the caller writes `give`/`copy`.
  `mut`/`!mut` go where they go on a declaration: before the name.
  A parameter with no type (`lent 'items'`) is legal: Cortex infers its
  best type from every call site in the program.
- **A function that can fail says so**: `--> bool or fail[full, bad_item: str.utf8]`
  or `--> fail[disk_full]`; see `failures.md` (decided 2026-09-19).
- **A variadic parameter is `...'name'`** (decided 2026-09-19): the rest
  of the arguments, any number including zero; last parameter only, one
  per function; composes with the verbs (`lent ...'parts'`). Model (b):
  every call site has a fixed shape, so Cortex compiles the function per
  shape — the same machinery generics will use — and with no type each
  argument keeps its own; the body's operations are the constraint,
  checked per call site (`sum[1, "two"]` fails to compile at that call).
  `...'nums': int32` restricts every argument. Inside the body the bundle
  walks like a list (`for 'n' in lend 'nums'`, `list.length[…]` is a
  compile-time constant there). `...` because it is the ellipsis — "and so
  on" — the mark C, Java, Go, JS, Swift all use; owner asked and chose it
  over `many` (reads like an array) and `vararg`. Open: a union-typed
  variadic (`int32 or bin16`) waits on union types. `examples/variadic.nrv`.
- **Unpacking a list into a call: `...'xs'`** (decided 2026-09-21).
  `sum[...'xs']` with `'xs'` holding `1, 2, 3` is `sum[1, 2, 3]`; it may
  sit among hand-typed values (`sum[10, ...'xs', 20]`) and a bundle may be
  passed on (`show["loud", ...'parts']`). Because a variadic is compiled
  per call shape, **the list's length must be known at the call**: a
  literal list, a `list[T][lo, hi]`, a variadic bundle. A plain list whose
  length Cortex cannot see (one read from input, one appended to without a
  bound) is a compile error at that call — "give it a bound or pass it as
  a list" — never a quietly built list-taking version (owner chose this
  over that). `...` is only meaningful as a call argument. **Owned
  elements** (text, lists, structs) going to an *owning* variadic need the
  verb said, as every owned hand-over does: `keep[...copy 'words']`
  duplicates each, `keep[...give 'words']` moves each out and leaves the
  list empty (a give never leaves holes). No verb, into a `lent ...`,
  lends each element; plain elements take no verb (they copy by
  themselves). Owner chose this over "always copy" and "always give".
  `examples/spread.nrv`.
- **The return type is `--> T`** after the parameter list (Rust's arrow),
  not `: T` — `:` attaches a type to a *name*, and a function's result is not
  the function's type. No arrow: Cortex picks the best return type from every
  `return` in the body; no `return` at all: the function returns nothing. A
  function declared *with* a return type must return a value on every path;
  falling off the end is a compile error.
- **The arrows are long: `-->` and `<--` are the main spelling** (owner,
  2026-10-03: "Long arrow is main in CyborgPL", then "yea, both"), wherever
  an arrow goes: a return type, a function type, an asm or `inline.MSL`
  binding, `print.weird`. **`->` and `<-` are other spellings** of them
  (the arrows were short until that day; the long ones came first as the
  aliases, "Implement the `-->` alias"), and so are **`>>` and `>>>`** of
  `-->`, and **`<<` and `<<<`** of `<--`, and the both-ways binding's main spelling is **`<-->`**, with `<->`, `<<>>`, `<<<>>>`, `<---->`, `<--|-->`, `<-|->`, `<<|>>` and `<<<|>>>` (owner, 2026-10-06: "yea.", "`<<>>` too", "Two long arrows", "`b`"); an
  arrow cannot name a Neuron's operator. `examples/long_arrow.nrv`,
  `examples/short_arrow.nrv`, `examples/chevron_arrows.nrv`.

## Calling (decided 2026-09-14)

A function is a member of a Neuron. In the Neuron where it is defined it is
called **bare**: `add[10, 20]`. From any other Neuron it is reached through
the boundary: `mathstuff::pow[2, 64]`, `std::print.stored.stdout[…]`, and
never bare (owner, 2026-09-23: no Neuron can offer a shortcut).

**Function names are bare at the definition too**: quotes mean *variable*,
and a function is not one. A function name is a plain identifier — no
spaces or symbols, not a keyword. Variables keep the anything-goes naming.

A call (`add[…]`) and a list index (`'items'[3]`) are told apart by
shape; a keyword and a call are told apart by the reserved-word list.

The operator words (`x`, `xx`, `mod`, `and`, `or`, `not`) are bare too, but
never collide with a call: an operator sits *between* operands and is never
followed by `[`. They may be function names; they are not reserved.

## Call

Arguments are comma-separated values in one `[ ]`, positionally matched to
the parameters. A call is a value and stands bare wherever a value goes:
`let 'sum' := add[10, 20];`, `let 'y' := add[1, 2] times 3;`,
`if is_even['n'] { }`. A call as a statement: `shout[lend.mut 'u'];`.
(Until 2026-09-19 the outer bracket was mandatory — `:= [add[10, 20]]`.)

## Return

```
return 'a' plus 'b';
return "hello " ++ 'name';    # text joins with ++ (2026-09-19)
return;                       # in a nothing-function
```

## Entry point: `MAIN { }` (decided 2026-09-14, kept 2026-09-19)

```
MAIN {
    let 'sum' := add[10, 20];
    cord::print.useful["sum: ", 'sum', newln];
}
```

`MAIN` is the program's block — not a function. No `fn`, no return type,
no `return`. When the block ends, the process exits 0; that is the one
implied thing in the language. Any other exit code is stated where it
happens: `std::exit[1];`, typed `uint8` because an exit status is 0–255.

`MAIN` is **unquoted** because it belongs to the language, not the user. It
cannot be called: it is not a function. It is the single root of Cortex's
whole-program analysis and owns the program's top-level values. Next to
`fn` it is the only shouting word in a file — the owner kept it for exactly
that reason: its distinctness is the feature, and Rust's `fn main` being an
ordinary callable function was the thing disliked.

A Neuron with a `MAIN` is a program; without one, a library. A Neuron another
depends on may have one too, run only when that Neuron is run itself
(owner, 2026-09-29).

**Command-line arguments arrive as C has them** (owner, 2026-09-29:
"low-level shit, decided" — "we're at Rust, C, C++ level"):

```
MAIN ['argc': int32, 'argv': ptr[ptr[uint8]]] {
    no_check let 'first' := ptr.read[ptr.at['argv', 1]];     # the first word after the program's name
    …
}
```

Exactly these two types (the names are free); the program's name comes
first, each word ends in a NUL, and `argv[argc]` is a null pointer. Reading
them is C's world, so `!check`. Anything friendlier — a list of text,
flags, `--help` — is written in Nerve on top, in std or a Neuron. The
2026-09-14 line here said the arguments "come from the std Limb"; the
owner put the door in the base language instead. Proposed alternatives: a
ready `list[str.utf8]` (design claude's recommendation); `std::args[]`;
nothing until std. `examples/args.nrv`.

History: first decided as `func.int32 MAIN` with a mandatory `return`, then
simplified the same day; `func` became `fn` on 2026-09-19.

## `[ ]` — one separator

- **Comma** separates *values*: `[10, 20]`. Two values with nothing between
  them are an error. (Until 2026-09-19 whitespace joined pieces into text;
  joining is now the explicit `++`.)

A bracket holding several comma-separated values is a **list**:
`let 'l' := [1, 2, 3];`. Function arguments are exactly that. A list of one
is written with a trailing comma: `[1,]`. A `[ ]` may sit inside a `[ ]` as
a value — `[[1, 2], [3, 4]]` is a list of lists.

## Brackets

- `[ ]` — builds a value (list, arguments, construction, index, range).
- `{ }` — body.
- `( )` — arithmetic grouping only: `'a' plus ('b' times 2)`.

## Numbers

Digit grouping uses `_`, never `,`: `1_000_000`. `1,000` is an error
(decided 2026-09-17); leading zeros are illegal.

## Still open

- The lifetime annotation's spelling.
