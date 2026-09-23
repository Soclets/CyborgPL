# CyborgPL syntax grammar

`cyborgpl.tmLanguage.json` is a TextMate grammar for `.cpls`. The website uses
it to highlight every code sample; it also works in VS Code and anything else
that reads TextMate grammars.

It was written from the oracle's lexer and parser — `oracle/src/lexer.rs` for
the token shapes and `oracle/src/parser.rs` for the keyword list and the type
spellings — so it follows the same rules the reference interpreter enforces,
including the 2026-09-19 syntax revision.

## What it knows

- Comments in all four forms: `#`, `#wl`, `#Nl`, `#Nld`.
- Text (`"…"`, where only `\"` and `\\` escape), characters (`` `c` ``), and
  quoted variable names (`'x'`).
- The bare `\n` newline piece, which is a value outside the string, not an
  escape inside it.
- Numbers with `_` grouping, integer and fractional.
- Every keyword in `parser.rs`'s `KEYWORDS`, including `var` and `func`, which
  were retired on 2026-09-19 and are marked deprecated so they read as wrong.
- Explicit type spellings: `int8`–`int128`, `uint8`–`uint128`, `bin16`–`bin64`,
  `deci32`–`deci128`, `str.utf8` and friends, `char.point`, `map.ordered`,
  `ptr.mut`, and the bare collections.
- Generic blanks, with or without an interface constraint (`<T>`,
  `<T: Ordered>`); struct, enum and interface names, including through a
  module (`counter.Count`); fields both where they are declared
  (`name: str.utf8`) and where one is built (`Locker[number = 1]`).
- Enum cases, both inside `enum Name { … }` and as the arms of `match` and of
  an `otherwise { … }` handler, where they name a case or a failure.
- Units after a number — durations (`5 seconds`, `1500 millis`) and a text
  bound's unit (`[1, 40 graphemes]`) — and `min` / `max` as the open end of a
  ranged type (`int32[1, max]`).
- Custom operators, where they are declared (`op dot […]`) and where they are
  used between two values (`'u' dot 'v'`).
- Modules before a dot (`counter.bump`), and a function named as a value
  (`first_by[…, by_number]`) — safe because variables are always quoted.
- Limb paths (`std::print.stdout`), conversions (`to.int32`), calls in
  brackets, and the narrowing dot.
- Ownership verbs, the check tiers, failure names after `fail`, and the word
  operators `x`, `xx`, `mod`, `and`, `or` plus the wrapping `x~` / `xx~` and
  the bitwise `and.bit` / `xor` / `shl` / `shr`.
- `inline.ASM { … }`, whose contents are left verbatim.

## What it relies on

Two facts the lexer and parser enforce, so the grammar can lean on them:
variables are always quoted, and module names are always lowercase.

One it cannot: **a bare capitalised word is treated as a type.** Every type in
the examples is capitalised and every function is not, but the language does
not require it. A function named `Shuffle` would be coloured as a type.

## Checking it

Tokenised against all 70 programs in `examples/`: every non-whitespace span
gets a scope, and each class was audited by hand — every word coloured as an
enum case is a real case or failure name, every capitalised word outside
comments and strings is a declared type, generic or interface, and every
bare word coloured as a function reference is one.
