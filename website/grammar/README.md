# Nerve syntax grammar

`nerve.tmLanguage.json` is a TextMate grammar for Zag's `.zg` (and `.zv`; once Nerve's `.nrv` and `.nrp`). The website uses
it to highlight every code sample; it also works in VS Code and anything else
that reads TextMate grammars.

It was written from the oracle's lexer and parser — `oracle/src/lexer.rs` for
the token shapes and `oracle/src/parser.rs` for the keyword list and the type
spellings — so it follows the same rules the reference interpreter enforces,
including the 2026-10-07 revision: `->` / `<-`, comparison in words
(`equal_to`, `at_most`…), `immut` / `no_check` / `no_cheat` / `no_shadow`, `newln`,
`cord::`, `shadow`, `is`, `pos` and labelled ranges.

## What it knows

- Comments in all four forms: `#`, `#wl`, `#Nl`, `#Nld` — and `#strict`, which is not a
  comment but the file's directive.
- Text (`"…"`, where only `\"` and `\\` escape), characters (`` `c` ``), and
  quoted variable names (`'x'`).
- The bare `\n` newline piece, which is a value outside the string, not an
  escape inside it.
- Numbers with `_` grouping, integer and fractional.
- `let` introduces a binding and `change` marks an assignment. The retired
  `var`, `func` and `gives` are marked deprecated. `!$` and `!mut` can modify a binding.
- Explicit type spellings: `int8`–`int128`, `uint8`–`uint128`, nonstandard widths
  such as `!std_int24`, `!std_u4` and `!std_uint256`, `bin16`–`bin64`, `brain16`
  (and `bfloat16` / `bf16`), Rust's names (`i8`–`i128`, `u8`–`u128`, `f16`–`f64`),
  `deci32`–`deci128`, `any`, `str.utf8` and friends, `char.point`, `map.ordered`,
  `ptr.mut`, and the bare collections.
- Generic blanks, with or without an interface constraint (`<T>`,
  `<T: Ordered>`, `<T>: any`, `<T>: any | @x`), on functions, operators and enums
  (`enum Maybe <T>: any { … }`); struct, enum and interface names, including through a
  module (`counter.Count`); fields both where they are declared
  (`name: str.utf8`) and where one is built (`Locker[number = 1]`).
- Enum cases, both inside `enum Name { … }` and as the arms of `match` and of
  an `otherwise { … }` handler, where they name a case or a failure.
- Units after a number — durations (`5 seconds`, `1500 millis`) and a text
  bound's unit (`[1, 40 graphemes]`) — and `min` / `max` as the open end of a
  ranged type (`int32[1, max]`).
- Custom operators, where they are declared (`op dot […]`, `op <+>`, `op div.floor`) and where
  they are used between two values (`'u' dot 'v'`, `'u' <+> 'v'`).
- A Neuron's precedence: `precedence.low_first` / `high_first` blocks of
  `level.when_chained.<mode> N { … }`, whose words and symbols name operators.
- The exact literal `*0.1` as one number.
- Modules before a dot (`counter.bump`), and a function named as a value
  (`first_by[…, by_number]`) — safe because variables are always quoted.
- Neuron paths (`cord::print.useful`), conversions (`to.int32`), calls in
  brackets, and the narrowing dot.
- Ownership verbs (and `&` / `&mut`, their other spelling), the check tiers including
  `NO_FUCKING_CHECKS`, failure names after `fail`, and the word operators `plus`,
  `minus`, `times`, `pow`, `div.floor` / `.ceil` / `.trunc` / `.near`, their `mod`
  counterparts, `and`, `or`, `and.then`, `or.else`, and `.wrap` forms (`div.trunc.wrap`
  too). The long arrows `-->` and `<--`. Bitwise
  operators include `and.bit`, `xor`, `shl` and `shr`. The comparisons `==`, `!==`, `<`,
  `>`, `</==`, `>/==`; the retired `/`, `<==`, `>==`, `+`, `+~` and `-~` are marked deprecated (a lone `-` is
  a negative's sign).
- `@x` on a returned borrow (`--> lent@x T`), `giveS` in a strict `match` (`gives` marked retired), and an operator
  handed over as a value (`op plus`, `op neg`, `op </==`).
- A loop's modifiers, strongest first: `for !c seq …`, `for para …` (`seq` there is a word,
  not the type); tests: `test fn`, `must`, `must.fail.reason`, `must.stop`; `print.weird`'s `<--`.
- `#strict`'s range labels, `range[from 1, to 4, by 1]`, and `pos`.
- `layout.C enum Key { a = 65 }`: a case and its number. `extern.C`, `layout.C`,
  `inline.S` / `inline.ASM` and `inline.MSL` read as one keyword, not a type.
- A library call through a built-in type as one name: `str.each.grapheme[…]`,
  `simd.load.some[…]`; `pub.neuron`; a module inside a module (`counter.inner.twice`).
- `inline.ASM { … }`, whose contents are left verbatim.

## What it relies on

Two facts the lexer and parser enforce, so the grammar can lean on them:
variables are always quoted, and module names are always lowercase.

One it cannot: **a bare capitalised word is treated as a type.** Every type in
the examples is capitalised and every function is not, but the language does
not require it, and the owner decided on 2026-09-24 that it won't. A function
named `Shuffle` would be coloured as a type.

## Checking it

`node website/grammar/audit.mjs` (with `website/zag`'s packages installed) tokenises
every program in `examples/` with the site's own highlighter. It prints every
non-whitespace span left without a scope — there should be none — and, for each scope,
every distinct word it was given, which is the list to read by hand for words in the
wrong class. `node website/grammar/audit.mjs <scope>` prints one class.

Since 2026-10-08 it also knows: `newln`; comparison words; `is`; `shadow`; `no_check`,
`no_cheat`, `no_shadow`; `not_std.int24`; `.ieee`, `.masked` and the bitwise `.wrap` forms;
`bare.never` / `case.insensitive [fn, var]`; `on.target` / `on.machine` and the target named;
the base's `optional` (`optional[int32]`, `optional.none`, `optional.value[…]`); `my.` paths; std's names written bare (`print.useful[…]`, `duration`); registers in an inline
block's `with […]`; `list.fill[value 0, count 10]`; names in any script (WSTN). The retired
spellings (`-->`, `==`, `</==`, `!mut`, `!$`, `$:=`, `nocheck`, `\n` outside text,
`stdkit::` / `std::`, `cca::`, `pub.limb`, `!std_int24`, `#strict`) are marked deprecated.

Since 2026-10-12 it also knows: loops and slices as sentences (`for 'i' from 1 to 10 by 1`,
`before`, `from pos 2, to pos 4`); labelled bounds (`int32[min 0, max 100]`); `incl.bytes[…]`,
`incl.str.utf8[…]`, `incl.C.system[…]` and `incl.C[…]`; C's types (`C.int`, `C.long_double`) and C
enums by their lowercase names (`color.BLUE`); `size.of[T]`; a struct built by its lowercase C
name (`buf[size = 42]`); and `case.any` / `case.exact` (`case.insensitive` / `case.sensitive`
marked deprecated).

Last run on 2026-10-15 against all 174 programs (`.zg`): no unscoped spans, and no old spelling marked.
Caught up then with 2026-10-14/15: `std::` is the standard library again (`cord::`, `stdkit::`, `cca::`, `kin::` are old),
`pub.strike` (`pub.neuron`, `pub.limb` old), the symbols as spellings of the words (`+ - * / % == != <= >= ** & && ||`,
`*const`/`*mut`/`*'p'`, `+=` and kin, `'i'++;`), `exactly`, `concat`, `str.empty`, the argument words (`key`, `part`, `at`,
`between` beside `value`, `count`, `from`, `to`, `by`), and `level 2 { plus, minus } left_first`.
