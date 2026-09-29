# The Synthia session — a brief

For the session that builds **Synthia**, CyborgPL's separate package
manager ("synthia claude"). Written 2026-09-29 by the design session, at
the owner's request. Read this first; then everything it points at. The
session is not started yet: the owner starts it, and then gives it its
share of the machine's cores.

## Who decides

The owner, Tankun Sriket, alone. You advise and ask — in plain text,
one question at a time, with a recommendation — and wait. Never use the
AskUserQuestion tool. Never fill in a default for an open decision: if
the docs do not say, it is a question, not a pick. CyborgPL is a hobby
language (owner, 2026-09-17): no timeline, no obligation. The design
session ("design claude") records decisions in this repo's
`DECISIONS.md`; a Synthia question goes to the owner, directly or
through it.

## Where you work

In **the Synthia repo**, `Cyborg-Assembly-Manufacturing-Site/Synthia`,
checked out at `/Users/ts/Synthia` (its own folder, so it has its own
Claude memory). You read this repo (`/Users/ts/CyborgPL v2`: `design/`,
`DECISIONS.md`, `examples/`) and never edit it. Commits: branch `main`,
author Tankun Sriket (repo-local config, set), subjects
`kind.folder.narrowing: sentence`, trailer `Co-Authored-By: synthia
claude <noreply@anthropic.com>`, `git pull --rebase` before pushing.

## What is decided

- **Two package managers** (owner, 2026-09-29): CCA has one built in
  (`cca add`, manifest `CCA.cplr` in CyborgPL syntax — compiler claude's).
  **Synthia is the separate one**, "fast, reliable, and easy-to-use".
- **Go** (owner, 2026-09-29). Its standard library covers what a package
  manager does: fetching, archives, hashing, running `cca`.
- **Its manifest is `Synthia.ttsr`** — *Tankun's TOML Syntax Ripoff* — in
  a syntax of Synthia's own, which "may not be like TOML" (owner). The
  syntax is **not designed**: that is the first question, and it is the
  owner's.
- **How it connects**: Synthia reads its manifest, fetches the Limbs,
  and calls `cca build …` saying where each is. CCA never reads
  `Synthia.ttsr` and never depends on Synthia (owner, 2026-09-29): CCA
  builds, runs and tests on its own.
- **What a manifest holds** (from earlier rulings): the Limb's name (never
  in source, 2026-09-23), its dependencies (never in source), the target
  (reproducible builds: never the host's), a version.
- **Licence**: Apache-2.0 WITH LLVM-exception, as CyborgPL (owner,
  2026-09-29: "same license").
- **Marks**: Synthia™. ™, never ®.

## What is open

The manifest's syntax; commands (`synthia build/final/run/test` are
named in `tooling.md` from before CCA had its own); lock files,
publishing, where Limbs are fetched from; how CCA takes the list of Limbs
(ask compiler claude, who owns CCA's command line). `design/tooling.md`,
`design/limbs.md`, `design/reproducible-builds.md`.

## Reaching the other sessions

By their permanent session ids (`CLAUDE.md` in this repo): design claude
`local_9078e904-c5c3-41ca-a97b-29bc24a56ed6`, compiler claude
`local_bd7d62d4-5ac0-4ce0-8f9a-174b30357045`, check claude
`local_ef7a2c2e-a6ee-44b5-a0bd-b5c9a62174ec`.
