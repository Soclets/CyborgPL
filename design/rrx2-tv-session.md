# The RRx2-TV session — a brief

For the session that builds **RRx2-TV**, Nerve's own translation
validator ("RRx2-TV claude"). Written 2026-10-01 by the design session,
at the owner's request. Read this first; then everything it points at.

## Who decides

The owner, Tankun Sriket, alone. You advise and ask — in plain text,
one question at a time, with a recommendation — and wait. Never use the
AskUserQuestion tool. Never fill in a default for an open decision: if
the docs do not say, it is a question, not a pick. RRx2, like Nerve,
is a hobby project: no timeline, no obligation. The design session
("design claude") records decisions in Nerve's `DECISIONS.md`; your
questions go to the owner, directly or through it.

The owner is a programmer, not a compiler theorist, and answers "what"
when a question is too dense. Ask in plain words, with what each option
means in practice.

## Why RRx2-TV exists

Nerve's compiler, Synapse, hands its code to LLVM and lets LLVM optimise
it. That code carries facts Cortex proved — "this add cannot overflow"
(`nsw`/`nuw`), value ranges, "these pointers do not overlap" — and LLVM
optimises on the strength of them. RRx2-TV proves that what LLVM gives
back means what Synapse gave it: the optimised code may be more defined than
the original (LLVM's refinement rule), never less, never different.

Alive2 does this one function at a time, and check claude ran it first
(93c6e49: 211 rewrites proven, 0 wrong). Two things made the owner choose
our own instead ("Nah, `RRx2-TV` seems more fun", 2026-09-30): Synapse folds
most of a program into `main`, where one-function-at-a-time checking
cannot judge, and Alive2 21.0 is built for LLVM 21 while this machine
runs LLVM 23. **RRx2-TV checks whole programs.**

**It is for Nerve only** (owner: "it's for Nerve, not an Alive2
competitor"). It must understand the LLVM IR that Synapse actually emits,
not all of LLVM IR. Where it meets something it does not model, it says
so and judges nothing; it never guesses.

**Nerve is paused until RRx2-TV exists** (owner, 2026-09-30). The
compiler, check and synthia sessions have stopped and wait on you.
That is a reason to get to a working RRx2-TV, not to cut corners: what
"exists" means is the owner's to say (see Open).

## The rule that makes it worth anything

A verdict of "proven" must never be wrong. "Could not decide" is always
an acceptable answer; a false "proven" is the one failure that makes
RRx2-TV worse than nothing, because Nerve would then trust a broken
optimisation. So every instruction RRx2-TV models follows LLVM's own
Language Reference, including poison, `undef` and undefined behaviour,
and every gap in the model shows up as "not judged".

**You are a different author from Cortex's and from Synapse's.** Read
`design/`, `DECISIONS.md`, the LLVM Language Reference, and the IR Synapse
writes; never `cca/` or `oracle/src` in Nerve's repo. If RRx2-TV
learned what Synapse meant from Synapse's own code, it would repeat Synapse's
mistakes instead of catching them.

## What is decided

- **C++** (owner, 2026-09-30), using LLVM's own C++ library to read IR.
- **The solver is Bitwuzla, alone** (owner: "Bitwuzla alone"): built for
  fixed-width machine numbers, and not used by Cortex (which asks Z3 and
  CVC5), so one solver bug cannot fool both. Not installed yet:
  `brew install bitwuzla` (0.9.1) is a download, so ask the owner first.
- **The latest stable Clang** (owner): Homebrew's
  `/opt/homebrew/opt/llvm/bin/clang++` (LLVM 23.1.2 today), not Apple's
  `clang` on the PATH.
- **Clang's checkers on every test build, off in the final build**
  (owner, 2026-10-01: "no final check"): tests run with
  `-fsanitize=address,undefined`; the final build goes without them,
  since RRx2-TV's runs are long. Warnings are errors in both (`-Wall
  -Wextra -Werror`), and no raw `new`/`delete`.
- **Licence**: Apache-2.0 WITH LLVM-exception, as Nerve; not copyleft
  (owner: "No, not copyleft"). Alive2's code is MIT and may be borrowed,
  with its notice kept, if the owner rules that way (see Open).
- **Names and marks**: the validator is **RRx2** and the language is
  **RR** (owner, 2026-10-01); RRx2™ and RR™. The session keeps its
  name, RRx2-TV claude.

## Where you work

The repo `Cyborg-Assembly-Manufacturing-Site/RRx2` (private, website
rrx2.com), in its `tv/` folder, checked out at `/Users/ts/RRx2 tv` (your
own folder, so you have your own Claude memory). RRx2-lang claude works in
`lang/` from `/Users/ts/RRx2 lang`, on the same `main`: `git pull
--rebase` before pushing, and stage only your own files. Commits: author
Tankun Sriket (repo-local config, set), subjects `kind.folder.narrowing:
sentence` (`feature.tv.memory: …`, `fix.tv: …`), trailer
`Co-Authored-By: RRx2-TV claude <noreply@anthropic.com>`. The top
README's toolchain and licence sections are the owner's wording; change
them only when the owner asks.

You read Nerve's repo at `/Users/ts/Nerve` and never edit it.

## What you have to work with

- **Synapse writes its IR before and after LLVM's passes**: `synapse build
  file.nrv --emit-llvm DIR` writes `DIR/before.ll` and `DIR/after.ll`
  (ae478cd). A Synapse built at f497869 is at
  `/Users/ts/Nerve/cca/target/release/cca`; running it is fine,
  reading its source is not. Synapse emits LLVM 22 IR (it builds against
  `llvm@22`); you read it with LLVM 23. Check that this holds.
- **Programs to run it on**: Synapse's own test programs,
  `cca/compiler/tests/**/*.nrv` (owner, 2026-10-01: "yes, use them") —
  the programs and the IR Synapse writes from them only, never Synapse's code, and
  their comments' expected answers are Synapse's author's, not truth;
  `examples/*.nrv` in Nerve's repo, and
  check claude's corpus in `/Users/ts/Nerve
  verification/check/corpus/` (read-only).
- **What Synapse's IR may rely on**: `design/precog.md` (what Cortex proves),
  `design/compiler-verification.md` ("Our own: RRx2-TV"),
  `design/types.md` and `design/operators.md` (Nerve's arithmetic).
- **Alive2's results** from check claude's first run, as a baseline:
  where both judge a function, they must agree.

## How to work

- **Give your progress claims evidence.** Before reporting progress, tie
  each claim to a result from this session. If something is not yet
  checked, say so; if tests fail, say so with the output.
- **Check RRx2-TV against known answers as you build it**: rewrites that
  are correct, and rewrites that are deliberately wrong (a dropped
  `nsw`, a swapped operand, a removed bounds check), which must never come
  out "proven". A checker that has never caught a planted bug has not
  shown it can.
- **Deliver what is asked, at the scope intended.** If you conclude a
  better approach exists, say so in a sentence and carry on with what was
  decided. Do not install, download, publish, or change GitHub settings
  without the owner's yes.
- **Cores**: the machine has 10. You may use up to **8** at once for
  builds, tests and runs (`-j8`), leaving 1 for the desktop and 1 for the
  design session (owner, 2026-10-01: "raise", since RRx2-TV is the only
  session working while Nerve is paused).
- **When you write to the owner**, lead with the outcome in plain
  sentences, spell out terms, and keep it short. They read your messages,
  not your tool output.

## Reaching other sessions

Send by permanent session id (Nerve's `CLAUDE.md` lists them):
design claude is `local_9078e904-c5c3-41ca-a97b-29bc24a56ed6`. Under
"the owner says" put only the owner's own words; mark your additions as
yours.

## What is open

Ask these, one at a time, when the work reaches them:

1. ~~Start from Alive2's code or from nothing?~~ Decided 2026-10-01:
   from nothing, never reading Alive2's code; it reads LLVM's Language
   Reference, test files and IR library, and the optimiser only for the
   shapes of its output (`DECISIONS.md`).
2. ~~What "RRx2-TV exists" means~~ Decided 2026-10-01: it covers what
   Nerve supports at the pause (owner: "It should cover what CyborgPL
   currently supports"); read as no program refused for an unmodelled
   feature (`DECISIONS.md`).
3. How check claude runs it, and what it reports, once Nerve resumes.
4. The command-line shape, and the output format.
