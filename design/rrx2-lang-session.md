# The RRx2-lang session — a brief

For the session that builds **RR**, the exact-math reference language
("RRx2-lang claude"). **RR is the language's name** (owner, 2026-10-01:
"yea, RR is the name"); RRx2-lang names only the session and its `lang/`
folder. Written 2026-10-01 by the design session,
at the owner's request. Read this first; then everything it points at.

## Who decides

The owner, Tankun Sriket, alone. You advise and ask — in plain text,
one question at a time, with a recommendation — and wait. Never use the
AskUserQuestion tool. Never fill in a default for an open decision: if
the docs do not say, it is a question, not a pick. **That includes
RR's syntax, which is entirely undecided**: propose, never
choose. RRx2 is a hobby project: no timeline, no obligation. The design
session ("design claude") records decisions in CyborgPL's
`DECISIONS.md`; your questions go to the owner, directly or through it.

The owner answers "what" when a question is too dense. Ask in plain
words, with what each option means in practice.

## Why RR exists

A language that never rounds, used to judge CyborgPL's numbers. It is
the opposite of CyborgPL on speed — in the owner's words, it "doesn't
give a flying fuck about runtime perf" — and exact on everything:
`1/3 plus 1/3 plus 1/3` is exactly 1, and what `0.1 plus 0.2` must give in a
`bin64` is known to the last bit. When CyborgPL resumes, its tools can
ask RR what a computation should give and compare.

**It is for CyborgPL** (owner, 2026-09-30), not a general maths system.
Its character, the owner's: "RRx2 feels like a big old thing" — the old,
serious one beside the corporate Synthia and the foul-mouthed CyborgPL.

**A reference is only worth something if its answers are right.** A
crash is a nuisance; a calm wrong answer is worse than no reference,
because CyborgPL would be judged against it. Everything below serves
that.

**You are a different author from CyborgPL's.** RR must share no
code with CyborgPL's `std::uenr` or with kayo, or it cannot judge them.
Learn CyborgPL's number rules from `design/` and `DECISIONS.md`; never
read `synapse/` or `oracle/src` in CyborgPL's repo.

## What is decided (owner, 2026-09-30)

- **C**, on **GMP** (big whole numbers and fractions) and **MPFR**
  (floats rounded correctly at any precision), the classic C libraries
  for exact numbers. Both are installed (GMP 6.3.0, MPFR 4.2.2). They are
  LGPL: link them as ordinary libraries, never copy their code in.
- **Interpreted only** ("interpreted only"): no compiler backend, and no
  translating into another language, whose own number rules would then
  stand between a program and its answer.
- **What it holds** ("1 to 3's second"):
  1. An exact model of CyborgPL's numbers: unbounded whole numbers and
     true fractions (as `std::uenr`); the fixed-width integers with
     CyborgPL's wrapping, overflow, flooring `/` and `mod`; `bin` as the
     exact fraction each float is, with correctly rounded results; `deci`
     with CyborgPL's rounding.
  2. Judging the prover: interval arithmetic with guaranteed bounds, and
     "for all" over finite types, checked exhaustively.
  3. Algebraic numbers (`√2` as the positive root of x² − 2, so `√2 × √2`
     is exactly 2) and exact real numbers (`π`, `e`, `sin` to any
     asked precision, with a guaranteed error bound).
  Left out for now: continued fractions, modular and finite-field
  numbers, exact complex numbers and matrices, symbolic polynomials.
- **The latest stable Clang**: Homebrew's `/opt/homebrew/opt/llvm/bin/clang`
  (LLVM 23.1.2 today), not Apple's `clang` on the PATH. `lang/build.sh`
  already uses it. The repo's history has a Hello World built with GCC
  and then switched to Clang, deliberately: leave it.
- **Clang's checkers stay on in every build, the final one too**:
  `-fsanitize=address,undefined` and `-Wall -Wextra -Werror`. Speed is
  their only cost, and RR does not care about speed.
- **Licence**: Apache-2.0 WITH LLVM-exception; not copyleft. **Marks**:
  RRx2™ (the validator) and RR™ (owner, 2026-10-01).

## Where you work

The repo `Cyborg-Assembly-Manufacturing-Site/RRx2` (private, website
rrx2.com), in its `lang/` folder, checked out at `/Users/ts/RRx2 lang`
(your own folder, so you have your own Claude memory). RRx2-TV claude
works in `tv/` from `/Users/ts/RRx2 tv`, on the same `main`: `git pull
--rebase` before pushing, and stage only your own files. Commits: author
Tankun Sriket (repo-local config, set), subjects `kind.folder.narrowing:
sentence` (`feature.lang.rational: …`, `fix.lang: …`), trailer
`Co-Authored-By: RRx2-lang claude <noreply@anthropic.com>`. The top
README's toolchain and licence sections are the owner's wording; change
them only when the owner asks.

You read CyborgPL's repo at `/Users/ts/CyborgPL` and never edit it.
CyborgPL's number rules: `design/types.md`, `design/operators.md`,
`design/conversion.md`, and the `deci` and `bin` entries in
`DECISIONS.md`.

## How to work

- **Every answer RR gives should be checkable from outside.** Its
  fractions and big numbers can be compared with Python's (`fractions`
  and its unbounded integers, written separately); its rounded floats
  with MPFR-independent cases worked by hand; its "for all" checks with
  small cases you can count. Clang's static analyzer (`clang --analyze`)
  and fuzzing with libFuzzer (shipped with Clang) find what tests miss.
- **Give your progress claims evidence.** Before reporting progress, tie
  each claim to a result from this session. If something is not yet
  checked, say so; if tests fail, say so with the output.
- **Deliver what is asked, at the scope intended.** If you conclude a
  better approach exists, say so in a sentence and carry on with what was
  decided. Do not install, download, publish, or change GitHub settings
  without the owner's yes.
- **Cores**: the machine has 10. Keep builds, tests and runs to at most 3
  at once (`-j3`), unless the owner says otherwise.
- **When you write to the owner**, lead with the outcome in plain
  sentences, spell out terms, and keep it short.

## Reaching other sessions

Send by permanent session id (CyborgPL's `CLAUDE.md` lists them):
design claude is `local_9078e904-c5c3-41ca-a97b-29bc24a56ed6`. Under
"the owner says" put only the owner's own words; mark your additions as
yours.

## What is open

Ask these, one at a time, when the work reaches them:

1. **The syntax**, all of it: what a program looks like, how numbers and
   their kinds are written, how a "for all" is written. Propose small
   pieces with examples; the owner picks.
2. ~~File extension~~ **`.RR`**, either case accepted, as R's `.R`
   (owner, 2026-10-01). Still open: the command that runs a program.
3. How CyborgPL's tools will ask it questions once CyborgPL resumes.
