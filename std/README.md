# std — Zag's standard library, in Zag

Written by std claude (`design/cord-session.md`). std is Zag calling the runtime (Rust) only for what
Zag cannot say yet (owner, 2026-09-14; again 2026-10-14, "(c)"). Every public item may be written with
`std::` or without it (owner, 2026-10-06). It was `cord/`, Nerve's cord, until the sweep to Zag's
names (2026-10-15).

## What is here

- `strike.zg` — the root: the `bare.optional` mark for every kind.
- `str.zg` — `std::str.contains`, `starts_with`, `ends_with`, `split`, `join`, `replace`
  (`design/strings.md`; the argument words are `spellings.zv`'s). They work on the bytes and call
  nothing in the runtime. Their `part`, `at` and `from` are texts of at least 1 byte, `str.utf8[min 1
  bytes]` (owner, 2026-10-14: "Error"), so an empty one is refused when building, or checked where it
  is passed if it is worked out at run time. The oracle's built-in std refuses it when the call runs:
  the same rule, met at a different moment.

A program uses this std by listing it with std's ID and a `path`, no `version` (owner, 2026-10-14):
`ricochet::dep[id = "fff4302dffc4a894a2bab6baf724eb21dfc05a2ad9818509d8001013d6761936", path = "…/std"]`.
The ID lives in that listing, so `Ricochet.zv` here carries none. Then every `std::` in the build is
this one, and what it does not define yet, `print` and `exit`, is still Zomp's Rust (owner,
2026-10-14: "(a)").

## How it is checked

- `examples/argument_words.zg`, unchanged, built by Zomp against this folder, prints its `.out`, every
  one of its sites proven. It is in Zomp's own tests against this folder
  (`zomp/compiler/tests/examples_nerve_cord.txt`).
- In the oracle, each of the six beside the oracle's own built-in std (a copy of this folder under
  another prefix), over 820 cases (empty text, separators at the ends and side by side, overlapping
  finds, `é`, `🍎`), printing the same; and the same cases built by Zomp against this folder, written as
  a user writes them (`std::str.split[lend 's', at lend 'p']`), printing what the oracle's built-in std
  prints.
- Through Zomp, with the text read from the command line, so the prover has to prove std's sites from
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
