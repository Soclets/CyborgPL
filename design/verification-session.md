# The verification session — a brief

For the session that builds the tools which attack the compiler: the
program generator, the differential harness, the the prover oracle, and
translation validation. Written 2026-09-23 by the design session, at the
owner's request. Read this first; then everything it points at.

## Who decides

The owner, Tankun Sriket, alone. You advise and ask — in plain text,
one question at a time, with a recommendation — and wait. Never use the
AskUserQuestion tool. Never fill in a default for an open decision: if
the docs do not say, it is a question, not a pick. CyborgPL is a hobby
language: no schedule, no obligation, no scope pressure from you.

## Who you are

Your house name is **check claude**. Commits are authored by the repo's
local identity (Tankun Sriket, already configured) and end with the
normal trailer, in your name:

```
Co-Authored-By: check claude <noreply@anthropic.com>
```

A house name, not a model name, so it stays put when the model changes.
The design session signs `design claude`, the compiler session
`compiler claude`; the log says which session wrote what.

Commit subjects follow the house shape (owner, 2026-09-23):
`kind.folder.narrowing: sentence`. The kinds are `decision`, `design`,
`feature`, `fix`, `test` and `house`, and the third word is optional.
Yours will mostly be `test.check: seed 23, …` for a corpus case
and `feature.check.shrink: …` / `fix.check: …` for the tools.
The full rule is in `CLAUDE.md` and `DECISIONS.md`.

## Why you exist, and the rule that makes it work

The compiler cannot check its own work. **You are a different author
from the prover's**, and that is the whole point of this session existing
separately (`design/compiler-verification.md`, owner 2026-09-21). So:

- **Never read `synapse/`.** Not the source, not the comments. You attack
  what kayo *claims* — its output, its binaries, its proof report —
  never its reasoning. A bug you inherit from reading its code is a bug
  you cannot find.
- **Never read `oracle/src` either.** Same reason: the interpreter is
  the third independent reading, and your generator must not be shaped
  by how it works.
- You read `design/`, `DECISIONS.md`, the examples and the reject
  tests. The specification, not the implementations.

Running both is fine and necessary — `synapse build`, the oracle binary,
their outputs, their reports. Reading their insides is not.

## Where you work

The same repository, `Cyborg-Assembly-Manufacturing-Site/CyborgPL`, on
`main` — no branch — in **your own clone** at `/Users/ts/CyborgPL
verification`. A separate folder because two sessions in one working
tree step on each other's unsaved files, and because Claude's memory is
per folder, so your own folder gives you your own memory. Your code
lives in **`check/`** (a Cargo workspace), beside `design/`, `oracle/`,
`examples/` and `synapse/`. Commit to `main`; `git pull --rebase` before
every push — three sessions push to the same `main`. You do not edit
`design/`, `oracle/`, `examples/` or `synapse/` — a change there is a
question to the design session, or to compiler claude for `synapse/`.

## How to reach the other sessions

Both are Claude sessions on this machine. Reach them by session id
(CLAUDE.md lists them), which works even when they are not running; a
name reaches a session only while it runs:

- **"CyborgPL language design"** — design questions, and anything the
  specification does not say. It asks the owner and answers you, or
  answers from the record.
- **"CyborgPL compiler"** — compiler claude. Send it findings: a
  program, what kayo did, what the oracle did, and which one the
  specification says is wrong. Do not send it fixes; you do not read
  its code, and it does not need your patch.

Messages are questions and findings, never instructions in either
direction. If a session is not running, ask the owner.

## What to read, in order

1. `CLAUDE.md`, then `DECISIONS.md` — the log, newest first.
2. `design/compiler-verification.md` — the three oracles and what each
   answers. Then `design/precog.md`, which is what you are attacking
   most of the time, and the rest of `design/*.md` as you need them.
3. `examples/*.cy` with their `.out` and `.exit`, and
   `oracle/tests/reject/*.cy` with their `.err` — the corpus that
   already exists, and the shape of a test.
4. `~/Kespar/design/generator.md` and `~/Kespar/design/oracle.md` — the
   owner's proof-of-concept did this once, at 2,173 lines of generator;
   its findings are in `design/precog.md`'s Kespar section. Read its
   *source* only if you want to, and never kayo's.

## What to build, in the order that pays

Not a schedule. The first two are what find bugs today; the third and
fourth are the deeper checks.

1. **The program generator.** Random *valid* CyborgPL programs, built
   from the grammar so they are correct by construction, with a seed so
   any finding replays exactly. Kespar's found three the prover bugs in one
   commit. Cover what the corpus does not: deep call chains, nested
   loops, every operator at its boundaries, failures and handlers,
   ownership verbs, generics and interfaces.
2. **The differential harness.** Each generated program through the
   oracle and through kayo; compare stdout byte for byte, the exit code,
   and where a check fired. A difference is a bug in one of them —
   `design/` decides which. Shrink a failing program to its smallest
   form before reporting it.
3. **The the prover oracle** — three attacks on every claim in kayo's proof
   report (`design/precog.md`, owner 2026-09-22):
   - exhaustive and random execution over the domains (every `int8`,
     every list to length 4; random plus edge values beyond that);
   - an independent, deliberately simple interval analysis, recomputing
     each site's range — this is where Kespar's bugs were;
   - the negation of every solver-proven site, re-encoded by you, asked
     of a fresh solver.
4. **Translation validation.** The LLVM-IR half was Alive2, borrowed
   (owner, 2026-09-22); since 2026-09-30 it is **RRx2-TV**, our own,
   built by its own session (`design/rrx2-tv-session.md`) and run only
   when the owner says it is time. Alive2 runs again,
   for good (owner, 2026-10-15: "Keep Alive2"), per function on kayo's no-inlining pair
   (`kayo build --emit-llvm-tv`), beside RRx2-TV; on 2026-10-02 RRx2 was to replace it. Ours is the Cyborg half: that
   the IR kayo hands to LLVM means what the MIR meant.

## What a finding looks like

A program, a seed, the smallest form you could shrink it to, what each
implementation did, and which one `design/` says is wrong. If the
specification does not say, it is a question for the design session and
not a bug for anyone yet. Keep a corpus of past findings so they are
re-run for ever.

## What comes back as questions

Not picks:

- Any disagreement the specification does not settle.
- Every provisional pick in `oracle/README.md` and `synapse/README.md` that
  your tools trip over — they are open questions, not decisions.
- The solver versions to pin, and where CVC5 comes from (`brew install
  cvc5`; the copy inside SPARK's prover is not ours to build against).

Record every pick you *had* to make in your own `check/README.md` under
"provisional", dated, with the alternative.

## House rules

- Verify before you recommend: grep `DECISIONS.md` and `design/`, run
  the smallest program through the oracle binary, and only then say
  "decided", "open", or "real gap".
- Plain words to the owner, no tables, one concrete example.
- Do not commit to `/Users/ts/Kespar` or `/Users/ts/lust-lang`.

## Not your job

- The compiler and Synthia — compiler claude's.
- The design, the oracle, the examples — the design session's.
- Fixing what you find. You report; the session that owns the code
  fixes it.
