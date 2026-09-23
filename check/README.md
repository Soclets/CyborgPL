# check — the verification session's tools

check claude's code (`design/verification-session.md`). It attacks what
CCA and the oracle *claim* — their output, exit codes, messages and
reports — and never reads `cca/` or `oracle/src`.

```
cd check && cargo build --release        # needs oracle/ and cca/ built with --release
./target/release/check run --cases 1000 --keep-going
./target/release/check corpus
```

## What is here

- **`gen/`** — the program generator. `seed → program` is a pure function
  (SplitMix64, no clock, no machine state), so any finding replays from its
  seed and size. Programs are valid by construction: every name declared
  before use, types matched, every loop bounded, and every operation kept
  inside its type by the generator's own simple interval bookkeeping, which
  folds an operand into range with `lo + (e mod w)` — a plain CyborgPL
  guardrail — where it could overflow. A few operations per thousand are
  left unguarded on purpose (`--risk`, per mille), so the engines are also
  compared on checks that fire. It writes: `int8`…`int64`, `uint8`…`uint64`,
  `bool`; `let` (plain / `mut` / `immut`), assignment, `if` / `else if` /
  `else`, `for` over `range` with every step sign (and `perm`), `while` and
  `loop` with counters, `break` / `continue` / early `return`, blocks,
  `check` / `nocheck` on statements and blocks, functions with typed
  parameters and returns calling one another, `+ - x / mod xx`, unary `-`,
  bitwise `and.bit or.bit xor not.bit shl shr`, wrapping `+~ -~ x~`,
  comparisons, `and or not`, `to.<int>[…]` narrowing and widening, range
  types on names, parameters and returns (`int32[0, 100]`, `int32[1, max]`,
  `int32[min, 100]`), and both parenthesised and "loose" spellings of the
  precedence rules.
- **Printing** (owner, 2026-09-23): every program carries its own printer in
  the base language — digits into a `list[uint8]`, then C's `write` under
  `nocheck` — varied by seed, compared like everything else.
- **Input** (owner, 2026-09-23): `--inputs K` makes MAIN read K values as the
  lengths of environment variables `CHK_IN1…` through `extern` `getenv` +
  `strlen`. The harness picks four input sets per program from the seed
  (all zero, then edges and random lengths); a finding records the
  variables and their lengths. Program runs see only those variables.
- **`harness/`** — the `check` binary. For each program and input set: the
  oracle binary; `cca build` and the binary it writes. Compared:
  - both ran → stdout byte for byte, both exit 0;
  - both stopped at run time → the same line, the same stdout before it;
  - CCA refused at compile time at line L → the oracle must stop at line L
    (a check that fails on known values is a compile error at the line,
    `design/precog.md` layer 1);
  - CCA's "not yet" → counted as coverage, never a finding.
  Findings are grouped by kind and by both engines' messages with numbers
  removed, so one root cause shows as one line. Each group's first seed is
  shrunk (remove statements and functions, replace expressions with their
  parts or small literals), keeping a step only if the same kind of finding
  survives in the build that showed it. A candidate must first be well
  scoped and make the oracle behave as before (both cheap); its CCA build
  gets 30 s. A shrunk program carries its inputs as `# env` lines.
- **`corpus/`** — every past finding, re-run for ever by `check corpus`.
  A `# env NAME LENGTH` line gives an input set.

## Which build a run uses

Every program is built twice: `cca build` (the working build: unproven
sites keep a runtime check) and `cca build --assert-proofs`
(`design/precog.md` attack (d): every proven site keeps a run-time check
that exits 104 with `proof failed:`). Both are compared with the oracle;
an exit 104 is a finding of its own kind, `proof-failed`, the most
serious. Not the final build. A green run says nothing about sites that
no input reached — that is what attacks (a)–(c) are for.

## Known bugs steered around

`--avoid` keeps the generator away from bugs already reported, so they do
not drown new ones; their cases are in `corpus/`. `--avoid none` aims
straight at them. Default: `ranges`.

- `ranges`: CCA does not accept range types yet (c845110, told
  2026-09-23); the generator writes no range types.

- `shl-width`: `shl`, `not.bit` and the wrapping operators only on names
  and `to.T[…]` — no longer needed (CCA takes the 88da73f rule since
  f40f659), kept as an option.
- `u64-high` (uint64 values ≥ 2^63 handled as int64) and `range-edge` (a
  `for` whose last step passes the counter type's edge) — both fixed in
  CCA c36cfce; still available as `--avoid` items.
- `xx` is not generated unless `--pow`: CCA does not generate code for it
  yet.

## Harness hygiene

Each engine run gets its own process group and its own output files, and
the group is killed when the run ends. (Before this, a `cca build` stopped
at its limit left a solver running that wrote into the next run's stdout —
NUL bytes and false "output" findings, 2026-09-23.) The shrinker confirms
every step it keeps by running it alone; a step that disagrees only under
load is set aside in `work/flaky/` with both runs' details.

## Provisional

Picks this session had to make to keep moving. Each is a question, not a
decision; dated, with the alternative.

- **Engines run as processes, not linked in** (2026-09-23). Kespar linked
  its engines as libraries; here that would mean compiling against
  `cca/` and `oracle/src`, which this session must not read. Cost: a CCA
  build per case (≈ 0.1–0.5 s).
- **Wall-clock limits: 20 s per program run, 120 s per `cca build`**
  (2026-09-23), a safety net against a hang. A run past its limit is a
  finding (`…-timeout`); a build past it is counted as slow, not a finding,
  since compile time is not a concern (`design/core-features.md`). Neither
  engine offers a step budget to the outside yet; with one, the run limit
  would be a count, as `design/precog.md` asks of Precog. Alternative: no
  limit.
- **The oracle's refusals and its run-time stops are not told apart**
  (2026-09-23): it prints both as `file: line N: …` and exits 1. The
  harness only compares lines. Alternative: ask the oracle for a marker.
- **Input sets**: four per program, lengths from 0, 1, 2, 3, 7, 10, 127,
  128, 255, 256, 1000, 32767, 32768, 65536 or random below 70,000
  (2026-09-23). Alternative: exhaustive over small domains (attack (a)).
- **Generator sizes and weights** (statement counts, depth, how often each
  construct appears, the work budget of 20,000 statement-trips for MAIN and
  400 per function) are guesses (2026-09-23), to be tuned by what finds bugs.
