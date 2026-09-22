# The compiler session — a brief

For the session that builds CCA, the Cyborg Chassis Assembler, and
JetPack. Written 2026-09-23 by the design session, at the owner's
request. Read this first; then everything it points at.

## Who decides

The owner, Tankun Sriket, alone. You advise and ask — in plain text,
one question at a time, with a recommendation — and wait. Never use the
AskUserQuestion tool. Never fill in a default for an open decision: if
the docs do not say, it is a question, not a pick. CyborgPL is a hobby
language: no schedule, no obligation, no scope pressure from you.

## Who you are

Your house name is **compiler claude**. Commits are authored by the
repo's local identity (Tankun Sriket, already configured) and end with
the normal trailer, in your name:

```
Co-Authored-By: compiler claude <noreply@anthropic.com>
```

Push to `main` over HTTPS, as the repo does. The design session's
commits carry `Claude Opus 5`; yours carry `compiler claude`, so the
log says who wrote what.

## Where you work (owner, 2026-09-23)

The same repository, `Artificial-IntelligenceAI/CyborgPL`, on `main` —
no branch — in **your own clone** at `/Users/ts/CyborgPL v2 compiler`.
A separate folder for two reasons: two sessions in one working tree
would step on each other's unsaved files, and Claude's memory is kept
per folder, so a folder of your own gives you a memory of your own
(owner's point). Your code lives in **`cca/`** (a Cargo crate) and
**`jetpack/`**, beside `design/`, `oracle/` and `examples/`, so the
docs and the tests you build against are always the current ones and
the differential harness runs `oracle/` and `cca/` side by side. Commit
to `main`; `git pull --rebase` before every push — the design session
pushes to the same `main` from its own clone. You do not edit
`design/`, `oracle/` or `examples/` — a change there is a question to
the design session.

## How to reach the design session

The design session is another Claude session on this machine, listed
by `ListAgents` as **"CyborgPL language design"** (the name may carry a
`[ref]`; use it if two rows share the name). Send it a design question
with `SendMessage` — it asks the owner and answers you, or answers from
the record when the record already says. If it is not running, ask the
owner directly. Everything the design session knows is in this repo;
messages are for questions, never for instructions in either direction.

## What to read, in order

1. `CLAUDE.md`, then `DECISIONS.md` — the log, newest first. Every rule
   has an entry saying what the owner chose and what was proposed.
2. `design/core-features.md`, `design/syntax-sketch.md`, then every
   other `design/*.md`. `design/precog.md` last of those; then
   `~/Kespar/design/precog.md` and `~/Kespar/README.md` — the owner's
   proof-of-concept of Precog, built from these docs, finished. It is
   the working example of the method; its provisional numbers are
   open questions here too. Read Kespar's *source* only when you build
   the matching layer (it is 14,600 lines; the first compiler session
   spent most of its first 240k tokens on it — not needed on day one).
3. The oracle's **tests only**: `examples/*.cpls` with their `.out` (and
   `.exit`), and `oracle/tests/reject/*.cpls` with their `.err`. **Never
   read `oracle/src`.** The oracle and CCA are two independent readings
   of one spec; a bug in one must not be copied into the other. This is
   the rule that makes differential testing mean something.
4. `oracle/README.md` — its lists of what it enforces and what it had
   to pick provisionally. Its picks are not decisions; they are
   questions.

## What "done" means

- Every `examples/*.cpls` builds and produces its `.out` byte for byte,
  with its `.exit` — once `std::` exists; until then, the ones that do
  not use it.
- Every `oracle/tests/reject/*.cpls` is refused, with a message that
  says what the `.err` says — in your words, in the shape
  `design/diagnostics.md` gives; the `.err` is a substring of the
  oracle's one-liner, not a template.
- The oracle and CCA agree on every program the generator makes.
- When they disagree, `design/` decides which is wrong; when `design/`
  is unclear, the owner decides and a line is added.

## The shape, decided

- **Rust**, with `inkwell` on top, `llvm-sys` only for calls inkwell
  has not wrapped, C++ only in the small LLVM wrapper. The LLVM version
  follows inkwell. `design/implementation-languages.md`.
- **Three IRs**: your AST; a **MIR** that is a control-flow graph with
  explicit temporaries, checks, drops, lends and failure edges, whose
  types carry proven domains; LLVM's. **Precog is the last MIR pass**;
  MIR is frozen after it. Proof reuse is keyed on canonical MIR (spans
  in a side table), the Merkle hash of everything reachable, argument
  domains, target, and the Precog and solver versions. `design/precog.md`.
- **Both builds run Precog in full.** `jetpack build` inserts a runtime
  check at every unproven site; `jetpack final` refuses the program if
  any default-tier site is unproven. There is no build that skips it.
  `design/core-features.md`.
- **Diagnostics** in the labelled plain-text shape (`error` / `check` /
  `note`, `where`, `code`, `when`, `why`, `try`, `fix`), carets optional,
  `--message-format json` for editors. `design/diagnostics.md`.
- **CCA never runs what it builds.** Running, testing, `--input`, and
  the per-target `runner` are JetPack's. The host is looked at for
  cores and memory only; the target is a description in
  `JetPack.toml`. `design/tooling.md`.
- **Every proven fact goes to LLVM where LLVM benefits**, measured by
  your own bench, re-run per LLVM upgrade. `design/precog.md`.

## How it is built (owner, 2026-09-23)

**Precog is built in from the first commit, not bolted on later.** MIR
carries domains and check sites from day one; layers 1 and 2 run as the
pipeline grows; the proof-to-LLVM path is part of the lowering from the
start. (An earlier draft here proposed "a Precog that proves nothing"
as a first milestone; the owner withdrew it: "optimization shouldn't be
bolted on later, it should be while building.")

**No `std::` for now** (owner, 2026-09-23): the base language first.
CCA refuses `std::…` as "not yet"; your tests print through the base
language's own door, `extern fn write` under `nocheck`, as
`examples/ffi.cpls` does. The examples that print with `std::printu`
wait for std; the reject tests and the extern-based programs do not.
The std Limb's author and timing are open.

## What comes back as questions

Not picks. Send them to the design session or the owner:

- The memory layout of `str`, `list`, `map`, `channel` in the runtime
  (the oracle deliberately does not model it).
- The proof cache's on-disk form.
- Every provisional number in `design/precog.md` (set caps, trip
  budgets, unroll depths, `rlimit`, the digit threshold, the invariant
  template list, the character classes).
- The GPU launch threshold; the JSON schema; anything `design/` leaves
  unsaid.

Record every pick you *had* to make to keep moving in your own
`README.md` under "provisional", the way the oracle does — dated, with
the alternative — so it can be turned into a question later.

## House rules

- Verify before you recommend: grep `DECISIONS.md` and `design/`, run
  the smallest program through the oracle binary (`oracle/target/…/oracle
  file.cpls` — running it is fine; reading its source is not), and only
  then say "decided", "open", or "real gap". The design session learned
  this the hard way (2026-09-21, twice).
- Plain words to the owner, no tables, one concrete example.
- Do not commit to `/Users/ts/Kespar` or `/Users/ts/lust-lang`; they are
  other projects.

## Not your job

- The verification tools — the Precog oracle, translation validation
  (Alive2 plus our glue), the program generator, the differential
  harness — belong to a third session with a different author, so that
  Precog's author never checks Precog. `design/compiler-verification.md`.
- The design — the design session's. If a rule seems wrong, say so as a
  question; do not build around it.
