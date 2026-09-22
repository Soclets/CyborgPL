# CyborgPL v2

Design repository for **CyborgPL v2**, a new programming language. This is a
from-scratch design — **not** a continuation of v1
(`Artificial-IntelligenceAI/CyborgPL-legacy`). No v1 syntax carries over unless
the owner explicitly chooses it.

## Ground rules

- **Design authority is the owner's.** There is exactly one language-design
  decider: the owner. Engineers and AIs — this session included — advise and
  recommend; none decide syntax, naming, semantics, or architecture, and none
  fill in a default for an open decision. Ask in plain text and wait.
- **Never use the AskUserQuestion tool** in this project.
- Record decisions in `DECISIONS.md` as they are made, noting what the owner
  chose versus what was proposed.
- Stated identity (from the website hero): *"Very slow at the development,
  fast & safe at the final product."*
- **CyborgPL is a hobby language** (owner, 2026-09-17). No timeline, no
  obligation to build any part by any date, no team unless the owner feels
  like one. Estimates, hiring notes, and the review policy in `design/` are
  for if-and-when, not commitments. Do not push for schedules or scope.

## Sessions

- **Design session** ("design claude", this repo's default): decisions, `design/`, `DECISIONS.md`, the oracle. Commits carry `Co-Authored-By: design claude <noreply@anthropic.com>`.
- **Compiler session** ("compiler claude"): CCA and JetPack. Its brief is `design/compiler-session.md`; it works in its own clone of this repo at `/Users/ts/CyborgPL v2 compiler` (same `main`, no branch; its own folder so it has its own Claude memory), in `cca/` and `jetpack/`; both clones `git pull --rebase` before pushing; it reads the oracle's tests, never `oracle/src`, and never edits `design/`, `oracle/` or `examples/`. Commits carry `Co-Authored-By: compiler claude <noreply@anthropic.com>`.
- **Verification session** ("check claude"): the program generator, the differential harness, the Precog oracle, translation validation. Its brief is `design/verification-session.md`; it works in its own clone at `/Users/ts/CyborgPL v2 verification`, in `check/`. It reads `design/` and the tests, and **never `cca/` or `oracle/src`** — it is a different author from Precog's, which is what makes its checks mean anything. Commits carry `Co-Authored-By: check claude <noreply@anthropic.com>`.

## Layout

- `DECISIONS.md` — the running log of settled design decisions.
- `design/` — design documents, one topic per file, as they are written.
- `examples/` — `.cpls` programs using settled syntax only, each with its
  expected `.out` (and `.exit` if non-zero). Run by the oracle's tests.
- `oracle/` — the reference interpreter (Rust): the executable spec. It
  implements only what `design/` decides and refuses the rest. When a note
  and the oracle disagree, fix whichever is wrong and add a test. Its
  README lists every provisional pick it had to make; those are open
  questions, not decisions. `cd oracle && cargo test`.

## Git

- Branch: `main`. Author: `Tankun Sriket <tankunsriket63741ecff056.invalid>`
  (repo-local config, already set).
- Every commit ends with the trailer in the session's house name — `Co-Authored-By: design claude <noreply@anthropic.com>` from this session, `compiler claude` from the compiler session. A house name says which session wrote it and stays put when the model changes (owner, 2026-09-23).

## Related

- Website: `/Users/ts/CyborgPL v2 website` (`CyborgPL-v2-website`, private).
  Its docs/examples are placeholders to be filled in as decisions land here.
