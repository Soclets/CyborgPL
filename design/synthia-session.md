> **Abandoned 2026-10-22** (owner: "Synthia is abandoned now, CyJet is already friendly enough"): this brief is history; no Synthia session runs.

# The Synthia session — a brief

For the session that builds **Synthia**, Zag's separate package
manager ("synthia claude"). Written 2026-09-29 by the design session, at
the owner's request. Read this first; then everything it points at.

## Who decides

The owner, Tankun Sriket, alone. You advise and ask — in plain text,
one question at a time, with a recommendation — and wait. Never use the
AskUserQuestion tool. Never fill in a default for an open decision: if
the docs do not say, it is a question, not a pick. Zag is a hobby
language (owner, 2026-09-17): no timeline, no obligation. The design
session ("design claude") records decisions in this repo's
`DECISIONS.md`; a Synthia question goes to the owner, directly or
through it.

## Where you work

In **the Synthia repo**, `Cyborg-Assembly-Manufacturing-Site/Synthia`,
checked out at `/Users/ts/Synthia` (its own folder, so it has its own
Claude memory). You read this repo (`/Users/ts/Zag`: `design/`,
`DECISIONS.md`, `examples/`) and never edit it. Commits: branch `main`,
author Tankun Sriket (repo-local config, set), subjects
`kind.folder.narrowing: sentence`, trailer `Co-Authored-By: synthia
claude <noreply@anthropic.com>`, `git pull --rebase` before pushing.

## What is decided

- **Two package managers** (owner, 2026-09-29): Zomp has one built in
  (`synapse add`, manifest `Ricochet.zv` in Zag syntax — compiler claude's).
  **Synthia is the separate one**, "fast, reliable, and easy-to-use".
- **Go** (owner, 2026-09-29). Its standard library covers what a package
  manager does: fetching, archives, hashing, running `cca`.
- **Its manifest is `Synthia.ttsr`** — *Tankun's TOML Syntax Ripoff* — in
  a syntax of Synthia's own (owner). Its syntax is designed:
  `design/synthia-manifest.md` (owner, 2026-09-29), and its one open point
  is there.
- **Synthia is an automatic programmer** (owner, 2026-09-30): it does
  whatever a programmer using Zomp would do by hand, writing a `Ricochet.zv`
  for a strike that has only a `Synthia.ttsr`. Zomp never knows Synthia
  exists. The project's own `Ricochet.zv` goes beside `Synthia.ttsr`,
  rewritten from it each time, its first line saying it is generated
  (owner, 2026-09-30: a): `# Written by Synthia from Synthia.ttsr, and
  rewritten each time: edit Synthia.ttsr, not this file.` (owner: "yea"). A `Ricochet.zv` there without that line is not Synthia's: Synthia
  refuses to write and leaves it, and `synthia takeover` overrides (owner: "`a`,
  and there should be an override command"; then "`a`" for `takeover`).
- **How it connects**: Synthia reads its manifest, fetches the strikes,
  and calls `synapse build …` saying where each is. Zomp never reads
  `Synthia.ttsr` and never depends on Synthia (owner, 2026-09-29): Zomp
  builds, runs and tests on its own.
- **What a manifest holds** (from earlier rulings): the strike's name (never
  in source, 2026-09-23), its dependencies (never in source), the target
  (reproducible builds: never the host's), a version.
- **Licence**: Apache-2.0 WITH LLVM-exception, as Zag (owner,
  2026-09-29: "same license").
- **Marks**: Synthia™. ™, never ®.

## For now: Go only

The owner, 2026-09-29: "make it only do Go for now". Synthia's builds and
tests run Go and nothing else, and it never runs `cca`. So it has no core
bound (owner, 2026-09-29: "No core bounds, because Go only for now is
stated"): Go's bursts are short, while a `cca` build uses LLVM on every core
it can get.

## What design claude owes you

Word when a keyword is added to `oracle/src/parser.rs` (your copy of the
list is from 898ad6d).

## What is open

Commands other than `synthia show` (`synthia-manifest.md`) and
`synthia prepare` (`tooling.md`); `synthia build/final/run/test` are
named in `tooling.md` from before Zomp had its own; lock files,
publishing, where strikes are fetched from; how Zomp takes the list of strikes
(ask compiler claude, who owns Zomp's command line). `design/tooling.md`,
`design/limbs.md`, `design/reproducible-builds.md`.

## Reaching the other sessions

By their permanent session ids (`CLAUDE.md` in this repo): design claude
`local_9078e904-c5c3-41ca-a97b-29bc24a56ed6`, compiler claude
`local_bd7d62d4-5ac0-4ce0-8f9a-174b30357045`, check claude
`local_ef7a2c2e-a6ee-44b5-a0bd-b5c9a62174ec`. Yours is
`local_0ecdeff6-f87d-4d75-a395-6fc8add64f08`.
