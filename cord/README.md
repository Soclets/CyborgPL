# cord — Nerve's standard Neuron, in Nerve

Written by cord claude (`design/cord-session.md`). cord is Nerve calling Axon, the Rust runtime, only
for what Nerve cannot say yet (owner, 2026-09-14; again 2026-10-14, "(c)"). Every public item may be
written with `cord::` or without it (owner, 2026-10-06).

## What is here

- `neuron.nrv` — the root: the `bare.optional` mark for every kind.
- `str.nrv` — `cord::str.contains`, `starts_with`, `ends_with`, `split`, `join`, `replace`
  (`design/strings.md`; the argument words are `spellings.nrp`'s). They work on the bytes and call
  nothing in Axon.

Synapse does not load this folder as `cord::` yet: the owner ruled on 2026-10-14 that a Neuron listing
the Nerve cord by `path` gets it under `cord::`, and compiler claude is building that. Until then, the
built-in print and exit stay Synapse's Rust (owner, 2026-10-14: "(a)").

## How it is checked, until Synapse loads it

A copy under the prefix `cordtest` (listing Synapse's built-in cord for print, exit and lists), called
from small programs:

- in the oracle, each of the six beside the oracle's own built-in cord, over 856 cases (empty text,
  separators at the ends and side by side, overlapping finds, `é`, `🍎`), printing the same;
- through Synapse, the same 856 cases printing what the oracle's built-in cord prints;
- through Synapse, with the text read from the command line, so Cortex has to prove cord's sites from
  cord's own code rather than by running the program.

Synapse refuses `str` as a module name (it keeps the word reserved; the oracle does not), so the
Synapse copy names the module `text`. That is for compiler claude, not a change to cord.

## Provisional

Each is a choice the record does not settle, dated, with the alternative. Each is an open question.

- **2026-10-14 — an empty `at` (split) or `from` (replace) is refused through the signature**, typed
  `str.utf8[min 1 bytes]`: a literal `""` does not compile and any other text is checked where it is
  passed. This follows good practice 7: a public Neuron states its limits in its signatures. The
  alternative is the oracle's current reading: split stops the program ("an empty separator"), and
  replace puts `to` between every two characters and at both ends. Asked of the owner.
- **2026-10-14 — an empty `part` is in every text**: contains, starts_with and ends_with give `true`,
  as the oracle does. Asked with the above.
- **2026-10-14 — replace goes left to right and never looks inside what it put in**
  (`"aaa"` from `"aa"` to `"b"` is `"ba"`), as the oracle does.
- **2026-10-14 — bytes cut on a character's edge become text with `to.str.utf8 … otherwise`**, and the
  `otherwise`, which cannot happen, prints `cord: a bug: …` on stderr and exits 101, the code of a
  failed run-time check. The alternatives: a fallback value such as `""`, which would hide a bug, or a
  way to turn bytes known to be UTF-8 into text without a failure (an intrinsic, or something in the
  language), which nobody has asked for yet.
- **2026-10-14 — `Kinesin.nrp` has no `id`**: Synapse reads none yet, and how cord carries its registry
  ID (`fff4302d…1936`) belongs to compiler claude's loading work. Its `version` is the one Synapse
  knows, `2026-10-06T12:57:14+07:00`.
