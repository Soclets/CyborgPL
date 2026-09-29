# Tooling names

Owner, 2026-09-14.

- **Cyborg Chassis Assembler (CCA)** — the CyborgPL compiler (owner,
  2026-09-18; spelling of "Chassis" to confirm — owner wrote "Chasis").
  Not an assembler in the `as` sense; the owner accepts the possible
  confusion. The chassis is the frame; Precog is what is bolted to it.
- **Nowork** — the Lunae (formerly N%) compiler (owner, 2026-09-18): the iteration compiler
  that does no proving and no optimising — no work — and hands the program
  to CCA for the final product. See `scripting.md`.
- **Precog** — the "everything at compile time" technology: whole-program
  execution and exact analysis over sets of values (see
  `core-features.md`). The compiler knows what the program will do before it
  runs. Used as a noun everywhere: Precog finds an overflow, `check` asks
  Precog to verify a guardrail, a "Precog-proven" site carries no runtime
  check. The word is shared with unrelated products (an AI company, a
  vulnerability scanner); the owner's view is that people search "CyborgPL
  Precog", not "Precog".

- **Limb** — the unit of code distribution, exactly like Rust's *crate*:
  every library or program is a Limb, Synthia installs Limbs, and the standard
  library is one Limb among them, reached as `std.` in a chain. (A cyborg has
  limbs.)
- **Synthia** — the package manager and build tool: it *synthesises*
  everything (owner, 2026-09-23, on renaming from JetPack). Its manifest
  is **`Synthia.ttsr`** — *Tankun's TOML Syntax Ripoff* (owner,
  2026-09-29), in a syntax of Synthia's own, which "may not be like TOML".
  History: on 2026-09-23 it was `Synthia.cplm` (*CyborgPL Manifest*) with
  TOML inside, the TOML part bundled into the owner's "yes" to the
  extension; reopened on 2026-09-29, when the owner made two package
  managers. Proposed along the way: `Synthia.toml`; `.cpm`; `.cplm` kept
  for Synthia's own syntax (design claude); `.sss` (*Synthia Special
  Syntax*, taken by SugarSS, a CSS syntax); `.ttr` (the owner's, used by
  TETR.IO replays). `.ttsr` is free everywhere. What the syntax looks like
  is open; Synthia is not being built yet.

- **`.cpls`** — a CyborgPL source file (*CyborgPL Source*). `.cpl` is
  deliberately avoided.
- **CCA's package manager** (owner, 2026-09-29) — a second, built-in one,
  "because a hobby is supposed to be fun": `cca add …`, no name of its own.
  Its manifest is **`CCA.cplr`**, written in CyborgPL syntax on purpose:
  `.cplr` (*CyborgPL Related*) is the extension for CyborgPL-syntax files
  that belong to the tooling, and the file's name says which one it is.
  One plain `const` per setting (owner, 2026-09-29), the shapes from
  `cca`, a tiny built-in Limb only `CCA.cplr` can reach:

  ```
  const 'name' := "shop";
  const 'version' := "1.0.0";
  const 'dependencies' := [
      cca::Dep[name = "geometry", id = "3f9a…4a5b", version = "1.2"],
      cca::Dep[name = "money", path = "../money"],
  ];
  const 'targets' := [cca::Target[name = "aarch64-macos"],];
  ```

  Every name and type is checked against the settings CCA knows.
  **`name`, `version` and at least one target are required** (owner,
  2026-09-29: "yea"), as in `Synthia.ttsr`. `version` is any text, at most 256 bytes
  (owner: "same" as `Synthia.ttsr`). **A
  manifest is data only** (owner, 2026-09-29): anything that must be
  computed lives in **`build.cplr`**, a full CyborgPL program (functions,
  loops, a `MAIN`), which **never runs by itself** — a special command runs
  chosen ones or all of them, a dependency's included. Both package managers
  run it. The command is **`cca prepare`** (owner, 2026-09-29: b):
  `cca prepare geometry, money` runs those Limbs' `build.cplr` (names
  separated by commas, owner; a space after the comma or not),
  `cca prepare --all` runs every one; `synthia prepare` alike. Proposed
  alternatives: `build-script`, `run-build`. Compiler claude's four
  suggestions, accepted by the owner: `cca prepare` acts on the Limb in
  the current directory, or a folder given as `cca run` takes one; a name is
  a dependency's local name in that Limb's `CCA.cplr`, or the Limb's own
  name; a `build.cplr` runs in its own Limb's folder; `--all` runs
  dependencies before the Limbs that use them, each once, stopping at the
  first that fails. Open: what a `build.cplr` may produce or pass to the
  build, and `cca prepare` with no names (refused until ruled). A
  **`cca::Dep` is a name plus a `version`** (from the registry) **or a
  `path`** (a folder on this machine) (owner, 2026-09-29: a):
  `cca::Dep[name = "money", path = "../money"]`. The settings the oracle
  knows: `name`, `version`, `dependencies`, `targets` (a list of
  `cca::Target[name = "aarch64-linux", runner = "qemu-aarch64"]`: several
  targets, since CCA's package manager can do whatever Synthia can, owner
  2026-09-29).

  Synthia stays the separate tool, with `Synthia.ttsr` in its own syntax
  (above). What goes in
  `CCA.cplr`, and how the two relate, is open.
- **`.limb`** — a *published* Limb: the Limb's directory (its manifest +
  `.cpls` files) packaged as one file for Synthia to upload, download, and
  checksum. Same relationship as a Rust crate folder to its `.crate` tarball.
  Checked 2026-09-14: no existing `.limb` format is registered.

  A Limb can never be an opaque precompiled binary: whole-program analysis
  must see inside every Limb to prove ranges, borrows, and guardrails across
  the call boundary. A `.limb` therefore always contains analysable source
  (optionally plus a cached front-end result, an optimisation for later).

## Cross-compiling, running elsewhere (discussed 2026-09-21, proposal)

CCA compiles and stops; it never executes what it built. The target is
declared in the manifest (`Synthia.ttsr` or `CCA.cplr`), never sniffed from the host; the host is
looked at only for cores and memory (speed), never for meaning — the same
program with the same config gives the same proofs and the same binary on
a laptop or a farm. Proof-cache entries are keyed by (function bytes,
inputs, target). Running a binary for another target is Synthia's:

```
[target.aarch64-linux]
runner = "qemu-aarch64"          # an emulator
[target.riscv64-board]
runner = "ssh board-3 --"        # a real device
```

Native: no `runner`. `synthia test` / `synthia run` / the differential
harness go through it. (Cargo's `runner`.) Spelling open until Synthia is
designed. Also noted: the oracle should take the target's widths and
pointer size as parameters (it does not yet).

## `synthia run … --input "…"` (proposed 2026-09-22)

Feeds the given text to the program as its input, so a diagnostic's
`try:` line reproduces a counterexample with one paste. Spelling open
with the rest of Synthia.

## Open

- Synthia's own syntax for `Synthia.ttsr`, and how Synthia and CCA's package manager relate.
- **The Limb registry is an index only** (owner, 2026-09-29): small files naming each Limb's versions and linking to each version's release where its author keeps it; the registry never stores code, so it costs nothing to host (static files). A release lives **on GitHub**; the index keeps **a checksum of every release**; **registering a new Limb is a reviewed pull request, and a new version needs no review**; and **malware-free is not guaranteed**, which the registry says plainly. Open: how an update proves it comes from the Limb's owner. Its domain, when it exists: **`limbs.xxx`** (owner, 2026-09-29: "`limbs.xxx` it is, I'll just wait until there's a limbs registery"; $95 a year, so not bought yet). Looked at: `cpl-limbs.dev`, `limbs.network`, `limbs.wtf`, `limbs.fyi`, `limbs-reg`; `limbs.dev` and `limbs.io` are taken; there is no `.registry` ending.
