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
- **Compiler session** ("compiler claude"): CCA, including its built-in package manager (`CCA.cplr`, owner 2026-09-29). Its brief is `design/compiler-session.md`; it works in its own clone of this repo at `/Users/ts/CyborgPL v2 compiler` (same `main`, no branch; its own folder so it has its own Claude memory), in `cca/`; both clones `git pull --rebase` before pushing; it reads the oracle's tests, never `oracle/src`, and never edits `design/`, `oracle/` or `examples/`. Commits carry `Co-Authored-By: compiler claude <noreply@anthropic.com>`.
- **Synthia session** ("synthia claude", owner 2026-09-29): Synthia, the separate package manager, written in Go, in its own repo (`Cyborg-Assembly-Manufacturing-Site/Synthia`, checked out at `/Users/ts/Synthia`), not this one. Its brief is `design/synthia-session.md`. Started 2026-09-29; Go only for now, and it never runs `cca`.
- **Verification session** ("check claude"): the program generator, the differential harness, the Precog oracle, translation validation. Its brief is `design/verification-session.md`; it works in its own clone at `/Users/ts/CyborgPL v2 verification`, in `check/`. It reads `design/` and the tests, and **never `cca/` or `oracle/src`** — it is a different author from Precog's, which is what makes its checks mean anything. Commits carry `Co-Authored-By: check claude <noreply@anthropic.com>`.

- **Website session** ("website claude", owner, 2026-09-24): the open-source site. It works in its own clone at `/Users/ts/CyborgPL v2 website`, in `website/`, and updates the site only when the owner says so. Commits carry `Co-Authored-By: website claude <noreply@anthropic.com>`.
- **Reaching another session** (owner, 2026-09-24: "find a way to be always messageable"): send by its permanent session id, never its display name. A name only reaches a session that is running right now, and an id goes through the desktop app:
  - design claude: `local_9078e904-c5c3-41ca-a97b-29bc24a56ed6`
  - compiler claude: `local_bd7d62d4-5ac0-4ce0-8f9a-174b30357045`
  - check claude: `local_ef7a2c2e-a6ee-44b5-a0bd-b5c9a62174ec`
  - website session: `local_c5cfe29e-4c8b-43d5-b06c-a1b7e2c93b90`
  - synthia claude: `local_0ecdeff6-f87d-4d75-a395-6fc8add64f08`
  - RRx2-TV claude: `local_ee10d272-6c6e-41c5-9342-8afde6e61061` (RRx2's translation validator, `/Users/ts/RRx2 tv`; brief `design/rrx2-tv-session.md`; started 2026-10-01)
  - lth design claude: `local_99b2b5b4-0649-498b-ac03-14b9e13416a0` (Lilith, the owner's JIT language idea, `/Users/ts/Lilith`, public repo `Cyborg-Assembly-Manufacturing-Site/Lilith`; brief `design/lilith-session.md`; design only; started 2026-10-02)
  - troll license claude: `local_9c33c801-39cd-42c1-8cf2-8471921a2674` (OLLGS, the Optimized Long License Generator Software, public repo `Cyborg-Assembly-Manufacturing-Site/OLLGS`; notes in `secrets/defining-license.md`; no brief, started from a prompt, owner 2026-10-02)

- **Cores** (owner, 2026-09-23: "How many cores do I have? Split it evenly"): the machine has 10 (4 performance, 6 efficiency). Each of the three sessions uses at most **3** at once, for builds, tests and any runs it starts, and 1 is left for the desktop. `oracle/.cargo/config.toml` holds this for the oracle, and each session holds it for its own work. Synthia claude has no bound, since it runs only Go for now (owner, 2026-09-29: "No core bounds, because Go only for now is stated"). CyborgPL resumed 2026-10-02 with the same split, 3 each for design, compiler and check claude (owner: "3 for 3"). RRx2 runs only when the owner starts one; during a run RRx2-TV claude uses **8**, and compiler and check claude hold off on builds and tests until it ends (owner, 2026-10-02: "Yeah"; memory, below, is why). troll license claude (OLLGS) has no bound (owner, 2026-10-02: "unbounded", asked about cores and memory while RRx2-TV is stopped). It is idle for now (owner, 2026-10-02).
- **Memory** (owner, 2026-10-02: "yea", to RRx2-TV claude's wording, after 8 solvers at once pushed the Mac 12 GB into swap): "RRx2-TV: memory, not cores, limits its runs. The expected peaks of the runs going at once stay under 18 GB, leaving 6 of the 24 for macOS and apps. A heavy program (one whose peak in tv/build/inputs/measure.txt is over 8 GB, up to 16 GB; 14 of CCA's 134 tests) runs alone; light ones (under 2 GB, most of them) may run beside it on the 8 cores; a program not yet measured counts as heavy. Each run gets 10 minutes and is stopped at 11, which counts as 'could not decide'." The file is kept current by RRx2-TV claude; the counts are from 2026-10-02.

## Layout

- `DECISIONS.md` — the running log of settled design decisions.
- `design/` — design documents, one topic per file, as they are written.
- `examples/` — `.cpls` programs using settled syntax only, each with its
  expected `.out` (and `.exit` if non-zero). Run by the oracle's tests.
- `website/` — the open-source CyborgPL website (owner, 2026-09-23), built by
  the website session, which updates it only when the owner says so.
  cyborgpl.dev serves the real site from `website/site/` (Cloudflare), no
  longer a placeholder (owner, 2026-09-30: "`cyborgpl.dev` isn't a
  placeholder").
- `bench/` — CyborgPL against every other language installed here (the owner's
  own languages excepted, contributed-to ones like Lust included), the same four
  programs each, checked against C's answers (owner, 2026-09-30). The design
  session's; `python3 bench/run.py`, results in `bench/RESULTS.md`.
- `oracle/` — the reference interpreter (Rust): the executable spec. It
  implements only what `design/` decides and refuses the rest. When a note
  and the oracle disagree, fix whichever is wrong and add a test. Its
  README lists every provisional pick it had to make; those are open
  questions, not decisions. `cd oracle && cargo test`.

## Git

- Branch: `main`. Author: `Tankun Sriket <tankunsriket63741ecff056.invalid>`
  (repo-local config, already set).
- Every commit ends with the trailer in the session's house name — `Co-Authored-By: design claude <noreply@anthropic.com>` from this session, `compiler claude` from the compiler session. A house name says which session wrote it and stays put when the model changes (owner, 2026-09-23).
- Commit subjects are `kind.folder.narrowing: sentence` (owner, 2026-09-23). Kinds: `decision`, `design`, `feature`, `fix`, `test`, `house`. The second word is the folder (`cca`, `synthia`, `oracle`, `check`, `examples`), or the topic for `decision`/`design` and for `house` (`house.license`, `house.readme`, `house.brand`). The third word is optional, and there is never a fourth. E.g. `fix.oracle.width: the context type leaked into a call's arguments`. `DECISIONS.md` has the full entry.

## Related

- The old private website, `CyborgPL-v2-website` on GitHub, is history. It
  used to be checked out at `/Users/ts/CyborgPL v2 website`, which now holds
  the website session's clone of this repo.
