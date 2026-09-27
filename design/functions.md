# Functions

Decided 2026-09-14; spelling revised 2026-09-19 (`fn`, name-then-type
parameters, `-> T`).

```
fn add ['a': int32, 'b': int32] -> int32 {
    return 'a' + 'b';
}
'sum' := add[10, 20];              # this Limb: bare
'big' := mathstuff::pow[2, 64];    # another Limb: Limb::path
```

## Definition

```
fn count [lent 'text': str.utf8] -> int32 { … }     # looks at it
fn edit  [lent.mut 'text': str.utf8] { … }          # may modify it
fn keep  ['text': str.utf8] { … }                   # owns it
fn twice [immut 'n': int32] -> int32 { … }          # stated mutability
```

- **`fn`**, bare name, parameters in one `[ ]`, optional `-> type`, body in
  `{ }`. `pub fn` makes it visible to other Limbs (`limbs.md`).
- **Parameters are `'name': type`.** The ownership verb, when there is one,
  sits before the name and mirrors the caller exactly: the caller writes
  `lend 's'`, the parameter reads `lent 'text': …`; `lend.mut` ↔ `lent.mut`.
  An **owning parameter has no keyword** — the caller writes `give`/`copy`.
  `mut`/`immut` go where they go on a declaration: before the name.
  A parameter with no type (`lent 'items'`) is legal: Precog infers its
  best type from every call site in the program.
- **A function that can fail says so**: `-> bool or fail[full, bad_item: str.utf8]`
  or `-> fail[disk_full]`; see `failures.md` (decided 2026-09-19).
- **A variadic parameter is `...'name'`** (decided 2026-09-19): the rest
  of the arguments, any number including zero; last parameter only, one
  per function; composes with the verbs (`lent ...'parts'`). Model (b):
  every call site has a fixed shape, so Precog compiles the function per
  shape — the same machinery generics will use — and with no type each
  argument keeps its own; the body's operations are the constraint,
  checked per call site (`sum[1, "two"]` fails to compile at that call).
  `...'nums': int32` restricts every argument. Inside the body the bundle
  walks like a list (`for 'n' in lend 'nums'`, `list.length[…]` is a
  compile-time constant there). `...` because it is the ellipsis — "and so
  on" — the mark C, Java, Go, JS, Swift all use; owner asked and chose it
  over `many` (reads like an array) and `vararg`. Open: a union-typed
  variadic (`int32 or bin16`) waits on union types. `examples/variadic.cpls`.
- **Unpacking a list into a call: `...'xs'`** (decided 2026-09-21).
  `sum[...'xs']` with `'xs'` holding `1, 2, 3` is `sum[1, 2, 3]`; it may
  sit among hand-typed values (`sum[10, ...'xs', 20]`) and a bundle may be
  passed on (`show["loud", ...'parts']`). Because a variadic is compiled
  per call shape, **the list's length must be known at the call**: a
  literal list, a `list[T][lo, hi]`, a variadic bundle. A plain list whose
  length Precog cannot see (one read from input, one appended to without a
  bound) is a compile error at that call — "give it a bound or pass it as
  a list" — never a quietly built list-taking version (owner chose this
  over that). `...` is only meaningful as a call argument. **Owned
  elements** (text, lists, structs) going to an *owning* variadic need the
  verb said, as every owned hand-over does: `keep[...copy 'words']`
  duplicates each, `keep[...give 'words']` moves each out and leaves the
  list empty (a give never leaves holes). No verb, into a `lent ...`,
  lends each element; plain elements take no verb (they copy by
  themselves). Owner chose this over "always copy" and "always give".
  `examples/spread.cpls`.
- **The return type is `-> T`** after the parameter list (Rust's arrow),
  not `: T` — `:` attaches a type to a *name*, and a function's result is not
  the function's type. No arrow: Precog picks the best return type from every
  `return` in the body; no `return` at all: the function returns nothing. A
  function declared *with* a return type must return a value on every path;
  falling off the end is a compile error.

## Calling (decided 2026-09-14)

A function is a member of a Limb. In the Limb where it is defined it is
called **bare**: `add[10, 20]`. From any other Limb it is reached through
the boundary: `mathstuff::pow[2, 64]`, `std::print.stdout[…]`, and
never bare (owner, 2026-09-23: no Limb can offer a shortcut).

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
`'sum' := add[10, 20];`, `'y' := add[1, 2] x 3;`,
`if is_even['n'] { }`. A call as a statement: `shout[lend.mut 'u'];`.
(Until 2026-09-19 the outer bracket was mandatory — `:= [add[10, 20]]`.)

## Return

```
return 'a' + 'b';
return "hello " ++ 'name';    # text joins with ++ (2026-09-19)
return;                       # in a nothing-function
```

## Entry point: `MAIN { }` (decided 2026-09-14, kept 2026-09-19)

```
MAIN {
    'sum' := add[10, 20];
    std::printu.stdout["sum: ", 'sum', \n];
}
```

`MAIN` is the program's block — not a function. No `fn`, no return type,
no `return`. When the block ends, the process exits 0; that is the one
implied thing in the language. Any other exit code is stated where it
happens: `std::exit[1];`, typed `uint8` because an exit status is 0–255.

`MAIN` is **unquoted** because it belongs to the language, not the user. It
cannot be called: it is not a function. It is the single root of Precog's
whole-program analysis and owns the program's top-level values. Next to
`fn` it is the only shouting word in a file — the owner kept it for exactly
that reason: its distinctness is the feature, and Rust's `fn main` being an
ordinary callable function was the thing disliked.

A Limb with a `MAIN` is a program; without one, a library. Command-line
arguments come from the std Limb.

History: first decided as `func.int32 MAIN` with a mandatory `return`, then
simplified the same day; `func` became `fn` on 2026-09-19.

## `[ ]` — one separator

- **Comma** separates *values*: `[10, 20]`. Two values with nothing between
  them are an error. (Until 2026-09-19 whitespace joined pieces into text;
  joining is now the explicit `++`.)

A bracket holding several comma-separated values is a **list**:
`'l' := [1, 2, 3];`. Function arguments are exactly that. A list of one
is written with a trailing comma: `[1,]`. A `[ ]` may sit inside a `[ ]` as
a value — `[[1, 2], [3, 4]]` is a list of lists.

## Brackets

- `[ ]` — builds a value (list, arguments, construction, index, range).
- `{ }` — body.
- `( )` — arithmetic grouping only: `'a' + ('b' x 2)`.

## Numbers

Digit grouping uses `_`, never `,`: `1_000_000`. `1,000` is an error
(decided 2026-09-17); leading zeros are illegal.

## Still open

- The lifetime annotation's spelling.
