# std — CyborgPL's standard library, in CyborgPL

Written by std claude (`design/std-session.md`). std is CyborgPL calling the runtime (Rust) only for
what CyborgPL cannot say yet (owner, 2026-09-14; again 2026-10-14, "(c)"). Every public item may be written with
`std::` or without it (owner, 2026-10-06). It was `cord/`, Nerve's cord, until the sweep to Zag's
names (2026-10-15), and took CyborgPL's names in the next sweep (2026-10-16).

## What is here

- `kit.cy` — the root: the `bare.optional` mark for every kind.
- `str.cy` — `std::str.contains`, `starts_with`, `ends_with`, `split`, `join`, `replace`
  (`design/strings.md`; the argument words are `spellings.cyr`'s). They work on the bytes and call
  nothing in the runtime. Their `part`, `at` and `from` are texts of at least 1 byte, `str.utf8[min 1
  bytes]` (owner, 2026-10-14: "Error"), so an empty one is refused when building, or checked where it
  is passed if it is worked out at run time. The oracle's built-in std refuses it when the call runs:
  the same rule, met at a different moment. `std::str.at.byte[lend 's', pos 2]` is the byte at a
  position (a `uint8`), and `std::str.at.point[lend 's', pos 2]` the character (a `char.point`), each
  failing `out_of_range` past the last one (owner, 2026-10-17: str.at next). A position is
  `uint64[min 1]` (owner, 2026-10-17: "(a)"): a written `pos 0` is refused when building, and a position
  worked out while running is proven or converted where it is passed (`to.uint64[min 1]['i'] otherwise 1`),
  so `out_of_range` means only past the end.
- `CyJet.cyr` lists the `unicode` limb, std's first, since `char.point` is unicode's (owner, 2026-10-17:
  "(a)"). A program using std does not inherit it: `argument_words` lists no limb.

A program uses this std, the folder it is written in, by listing it as
`cyjet::dep.path.std["…/std"]`, a folder narrowed to std (owner, 2026-10-14; the spelling 2026-10-19). A
lone file built by KAYO uses it too, as the installed std's stand-in. Then every `std::` in the build is
this one, and what it does not define yet, `print` and `exit`, is still KAYO's Rust (owner, 2026-10-14:
"(a)").

## How it is checked

- `examples/argument_words.cy` and `examples/inventory.cy`, unchanged, built by KAYO against this
  folder, print their `.out`, every one of their sites proven. Both are in KAYO's own tests against this
  folder (`kayo/compiler/tests/examples_cyborgpl_std.txt`; `inventory +unicode`, as it uses `char.point`
  itself).
- In the oracle, each of the six beside the oracle's own built-in std (a copy of this folder under
  another prefix), over 820 cases (empty text, separators at the ends and side by side, overlapping
  finds, `é`, `🍎`), printing the same; and the same cases built by KAYO against this folder, written as
  a user writes them (`std::str.split[lend 's', at lend 'p']`), printing what the oracle's built-in std
  prints.
- Through KAYO, with the text read from the command line, so Precog has to prove std's sites from
  std's own code rather than by running the program: all of std's are proven.
- An empty `part`, `at` or `from` is refused when building if written, and stops the program where it
  is passed if worked out at run time (exit 101), as the oracle does.

## Provisional

Each is a choice the record does not settle, dated, with the alternative. Each is an open question.

- **2026-10-14 — replace goes left to right and never looks inside what it put in**
  (`"aaa"` from `"aa"` to `"b"` is `"ba"`); split keeps empty pieces (`",a"` at `","` is `str.empty`
  and `"a"`); join of an empty list is `str.empty`. All as the oracle does; shown to the owner
  unasked, 2026-10-14, and nothing came back against them.
- **2026-10-14 — bytes cut on a character's edge become text with `to.str.utf8 … otherwise`**, and the
  `otherwise`, which cannot happen, prints `std: a bug: …` on stderr and exits 101, the code of a
  failed run-time check. The alternatives: a fallback value such as `str.empty`, which would hide a
  bug, or a way to turn bytes known to be UTF-8 into text without a failure (an intrinsic, or something
  in the language), which nobody has asked for yet.
