# Functions

> Revised 2026-10-07: code here is in the new syntax; an old spelling in the prose is history. The old → new table is `revision-2026-10-07.md`.

Decided 2026-09-14; spelling revised 2026-09-19 (`fn`, name-then-type
parameters, `--> T`).

```
fn add [let $a: i32, let $b: i32] -> i32:
    ret $a + $b;
;fn add
let $sum := add(10, 20);              # this Kit: bare
let $big := mathstuff::pow(2, 64);    # another Kit: Kit::path
```

## Definition

```
fn count [lent $text: str.utf8] -> i32:
    …
;fn count     # looks at it
proc edit  [lent.mut $text: str.utf8]:
    …
;proc edit          # may modify it
fn keep  [let $text: str.utf8]:
    …
;fn keep                   # owns it
fn twice [let $n: i32] -> i32:
    …
;fn twice          # stated mutability
```

- **`fn`**, bare name, parameters in one `[ ]`, optional `--> type`, body in
  `{ }`. `pub fn` makes it visible to other kits (`kits.md`).
- **Parameters are `'name': type`.** The ownership verb, when there is one,
  sits before the name and mirrors the caller exactly: the caller writes
  `lend 's'`, the parameter reads `lent 'text': …`; `lend.mut` ↔ `lent.mut`.
  An **owning parameter has no keyword** — the caller writes `give`/`copy`.
  `mut`/`!mut` go where they go on a declaration: before the name.
  A parameter with no type (`lent 'items'`) is legal: the prover infers its
  best type from every call site in the program.
- **A function that can fail says so**: `--> bool or fail[full, bad_item: str.utf8]`
  or `--> fail[disk_full]`; see `failures.md` (decided 2026-09-19).
- **A variadic parameter is `...'name'`** (decided 2026-09-19): the rest
  of the arguments, any number including zero; last parameter only, one
  per function; composes with the verbs (`lent ...'parts'`). Model (b):
  every call site has a fixed shape, so the prover compiles the function per
  shape — the same machinery generics will use — and with no type each
  argument keeps its own; the body's operations are the constraint,
  checked per call site (`sum[1, "two"]` fails to compile at that call).
  `...'nums': i32` restricts every argument. Inside the body the bundle
  walks like a list (`for 'n' in lend 'nums'`, `list.length[…]` is a
  compile-time constant there). `...` because it is the ellipsis — "and so
  on" — the mark C, Java, Go, JS, Swift all use; owner asked and chose it
  over `many` (reads like an array) and `vararg`. Open: a union-typed
  variadic (`i32 or f16`) waits on union types. `examples/variadic.cy`.
- **Unpacking a list into a call: `...'xs'`** (decided 2026-09-21).
  `sum[...'xs']` with `'xs'` holding `1, 2, 3` is `sum[1, 2, 3]`; it may
  sit among hand-typed values (`sum[10, ...'xs', 20]`) and a bundle may be
  passed on (`show["loud", ...'parts']`). Because a variadic is compiled
  per call shape, **the list's length must be known at the call**: a
  literal list, a `list[T][lo, hi]`, a variadic bundle. A plain list whose
  length the prover cannot see (one read from input, one appended to without a
  bound) is a compile error at that call — "give it a bound or pass it as
  a list" — never a quietly built list-taking version (owner chose this
  over that). `...` is only meaningful as a call argument. **Owned
  elements** (text, lists, structs) going to an *owning* variadic need the
  verb said, as every owned hand-over does: `keep[...copy 'words']`
  duplicates each, `keep[...give 'words']` moves each out and leaves the
  list empty (a give never leaves holes). No verb, into a `lent ...`,
  lends each element; plain elements take no verb (they copy by
  themselves). Owner chose this over "always copy" and "always give".
  `examples/spread.cy`.
- **The return type is `--> T`** after the parameter list (Rust's arrow),
  not `: T` — `:` attaches a type to a *name*, and a function's result is not
  the function's type. No arrow: the prover picks the best return type from every
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
  arrow cannot name a kit's operator. `examples/long_arrow.cy`,
  `examples/short_arrow.cy`, `examples/chevron_arrows.cy`.

## `fn` and `proc` (owner, 2026-10-27; revised 2026-10-29)

A `fn` is a function in the maths sense: what it gives back depends only on what it is handed, and nothing else sees it
run. A `proc` may have effects. A `fn` calls only `fn`s; a `fn` value goes where a `proc` type is asked, never the reverse.

- **Effects**, which make a `proc`: printing, input, the clock, waiting, exiting (`std::exit`), a task, a channel, C or
  another language's code, a pointer.
- **Not effects**, which a `fn` may do:
  - change its own local variables (2026-10-27, "(a)");
  - change what it was lent, through `lent.mut` (2026-10-29, "(a)"): a `lend.mut` hands the value over and takes the
    changed one back, and while it is lent nothing else can see it, so it is a second output, not an effect.
    `fn append_point [lent.mut $out: std::ls[u32], let $c: u32]` and `fn Counter.next [lent.mut $c: Counter]` are
    functions (Mercury's unique modes, Lean's and Koka's in-place updates are the same idea);
  - fail (`-> T or fail[…]`): a function with no answer for some inputs is partial, still a function;
  - stop on a bug, `bug "…";` (2026-10-29).

`examples/lend_mut_is_pure.cy`, `examples/bug_stop.cy`.

## Calling (decided 2026-09-14)

A function is a member of a kit. In the kit where it is defined it is
called **bare**: `add[10, 20]`. From any other kit it is reached through
the boundary: `mathstuff::pow[2, 64]`, `std::print.stored.stdout[…]`, and
never bare (owner, 2026-09-23: no kit can offer a shortcut).

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
ret $a + $b;
ret "hello " concat $name;    # text joins with ++ (2026-09-19)
ret;                       # in a nothing-function
```

## Entry point: `MAIN { }` (decided 2026-09-14, kept 2026-09-19)

```
MAIN:
    let $sum := add(10, 20);
    std::print("sum: ", $sum, newln);
;MAIN
```

`MAIN` is the program's block — not a function. No `fn`, no return type,
no `return`. When the block ends, the process exits 0; that is the one
implied thing in the language. Any other exit code is stated where it
happens: `std::exit[1];`, typed `u8` because an exit status is 0–255. On a chip with no OS (`aarch64-none`)
the end calls the target's `exit` with 0 when the target names one, and otherwise the CPU sleeps there
(2026-10-09; 2026-10-26).

`MAIN` is **unquoted** because it belongs to the language, not the user. It
cannot be called: it is not a function. (Since 2026-10-26 a writer's function may be
named `MAIN` as a keyword may, reached only by its path, `my.MAIN()`; bare `MAIN` is
always this block.) It is the single root of the prover's
whole-program analysis and owns the program's top-level values. Next to
`fn` it is the only shouting word in a file — the owner kept it for exactly
that reason: its distinctness is the feature, and Rust's `fn main` being an
ordinary callable function was the thing disliked.

A kit with a `MAIN` is a program; without one, a library. A kit another
depends on may have one too, run only when that kit is run itself
(owner, 2026-09-29).

**Command-line arguments arrive as C has them** (owner, 2026-09-29:
"low-level shit, decided" — "we're at Rust, C, C++ level"):

```
MAIN [let $argc: i32, let $argv: ptr[ptr[u8]]]:
    no_check let $first := ptr.read(ptr.at($argv, by: 1));     # the first word after the program's name
    …
;MAIN
```

Exactly these two types (the names are free); the program's name comes
first, each word ends in a NUL, and `argv[argc]` is a null pointer. Reading
them is C's world, so `no_check`. Anything friendlier — a list of text,
flags, `--help` — is written in CyborgPL on top, in std or a kit. The
2026-09-14 line here said the arguments "come from the std Limb"; the
owner put the door in the base language instead. Proposed alternatives: a
ready `list[str.utf8]` (design claude's recommendation); `std::args[]`;
nothing until std. `examples/args.cy`.

History: first decided as `func.i32 MAIN` with a mandatory `return`, then
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

Since 2026-10-22 (owner): **`( )` runs something; `[ ]` describes or holds something.**

- `( )` — a call: `take_order(lend $items, 104)`. **The one exception is maths grouping**, which runs nothing:
  `$a + ($b x 2)` (owner, 2026-10-28: "There's an exception, though. Maths grouping").
- `[ ]` — a definition's parameters, a function's type, types, ranges, lists, indexes, building a struct, `fail[…]`
  lists: `fn add [let $a: i32, let $b: i32] -> i32:`, `std::ls[u8]`, `i32[min: 1, max: 99]`, `[1, 2, 3]`, `$xs[pos: 2]`,
  `Rect[w = 640, h = 480]`. A conversion has both: `to.i32[min: 1, max: 99]($text)`.
- A body is `name:` … `;kind name` (redesign three, 2026-10-28).

Before 2026-10-22, `[ ]` also carried a call's arguments and `( )` was arithmetic grouping only.

## Numbers

Digit grouping uses `_`, never `,`: `1_000_000`. `1,000` is an error
(decided 2026-09-17); leading zeros are illegal.

## Still open

- The lifetime annotation's spelling.
