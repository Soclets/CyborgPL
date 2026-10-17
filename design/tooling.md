# Tooling names

> Revised 2026-10-07: code here is in the new syntax; an old spelling in the prose is history. The old → new table is `revision-2026-10-07.md`.

Owner, 2026-09-14.

- **`KAYO\n`** — the CyborgPL compiler, written plainly **`kayo`** for the command, its folder `kayo/` and
  anything a machine types, as C# is `csharp` (owner, 2026-10-15: "`KAYO\n`?", then "yup"; from Valorant's
  KAY/O, the slash swapped for a newline escape). It was **Zomp** (2026-10-14), **Synapse** (2026-10-09;
  command `synapse` or `syn`, also `sng`, "Synapse Not GCC", and `suc`, which the CNC limb added), and **CCA**,
  the *Cyborg Chassis Assembler*, before that (owner, 2026-09-18). It names itself by its tools,
  `CyDT 1 KAYO\n` (`distribution.md`).
- **Nowork** — the Lunae (formerly N%) compiler (owner, 2026-09-18): the iteration compiler
  that does no proving and no optimising — no work — and hands the program
  to `kayo` for the final product. See `scripting.md`.
- **Precog** — the "everything at compile time" technology: whole-program execution and exact
  analysis over sets of values (see `core-features.md`, `precog.md`). The compiler knows what the program will
  do before it runs: Precog finds an overflow, `check` asks it to verify a guardrail, a proven site
  carries no run-time check. It is built into `kayo`, and `kayo prove` runs it alone (owner, 2026-10-16). It
  was Precog first, then **Cortex** (2026-10-09), then nameless in Zag (2026-10-14: "might wanna name it Zomp
  too"), and Precog again with CyborgPL (owner, 2026-10-15: "Actually, the CyborgPL prover's name is even greater than
  CyborgPL prover, it's `Precog`").

- **the runtime** — the `no_std` Rust library linked into every program (thread pool,
  check-failure handler, program entry, the OS edge). It has no brand name (owner, 2026-10-14:
  "Runtime could just be normal name"); it was **Axon** (2026-10-09).

- **kit** — the unit of code distribution, exactly like Rust's *crate*: every library or program is a kit, and
  the standard library, `std`, is one kit among them (owner, 2026-10-15: "Should we make plugin limb, instead of
  core? And a library is a kit?"). "Library" and "program" are words for how a kit is used, not names in code.
  Its root is `kit.cy`, and `pub.kit` makes an item visible to other kits. It was a **strike** (2026-10-15), a
  **Neuron** (2026-10-09), briefly a Cell (2026-10-14), and a **Limb** before that; `limb` then meant a library.
- **limb** — a compiler extension, such as `unicode`, listed as `limbs = [cyjet::limb[…]]` (owner, as above). It
  was a **plugin** (2026-10-15), a **lobe** (2026-10-09), briefly a Coil, and a *core* before that.
- **the base** — the language with no limb: lowercase, no ™ (owner, 2026-10-15: "Signatar could just be called
  base, not trademarked too"). It was **Ground** (2026-10-14), the **Brainstem** (2026-10-09) and Signatar.
- **`std`** — the standard library, a kit (owner, 2026-10-16: "`std`", over `stdkit`). It was `cord`
  (2026-10-09) and the std Limb.

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

- **`.cy`** — a CyborgPL source file, code and only code (owner, 2026-10-16: "`.cy` it is"); a file that is one
  CyborgPL value and never runs ends **`.cyr`** (owner, 2026-10-16: "`.cyr` it is", over `.cystatic`, "CyborgPL
  related"). They were `.zg` / `.zv` (Zag), `.nrv` / `.nrp` (Nerve) and `.cpls` / `.cplr` before that. `.cpl`
  is deliberately avoided (a Windows Control Panel file).
- **`synthia sync`** (owner, 2026-10-06, via synthia claude): writes the
  project's `CyJet.cyr` from `Synthia.ttsr`, one for each needed kit that
  has only a `Synthia.ttsr`, and the renamed copies in `.synthia/`, after
  checking the whole tree of path dependencies, so `synapse build` follows
  knowing nothing of Synthia; a later `synthia build` would run it first.
  A needed kit on this machine with only a `Synthia.ttsr` gets its
  `CyJet.cyr` in its own folder, never a copy (owner: "`a`").
- **CyJet, CyborgPL's built-in package manager** (owner, 2026-10-15: "CyJet it is"; it was Ricochet,
  2026-10-15, briefly Zopm, and Kinesin, 2026-10-09; it has no command of its own yet, preparing is
  `kayo prepare`) — a second, built-in one,
  "because a hobby is supposed to be fun".
  Its manifest is **`CyJet.cyr`** (was `Ricochet.zv`, `Kinesin.nrp`, `CCA.cplr`), written in CyborgPL syntax on purpose:
  `.cyr` is the ending for CyborgPL-syntax files
  that are one value and never run, and the file's name says which one it is;
  its types are `cyjet::` (2026-10-16; was `ricochet::`, `kin::`, and `cca::` before that).
  **One value, `cyjet::manifest[ … ];`** (owner, 2026-10-06: "`c`. It may
  not look as complex, but at least it's not much of a roleplay"; from
  2026-09-29 to then, one plain `const` per setting), the shapes from
  `cca`, a tiny built-in kit only `CyJet.cyr` can reach:

  ```
  cyjet::manifest[
      name = "shop",
      prefix = "shop",
      version = "1.0.0",
      limbs = [cyjet::limb[id = "5d0e846b…44b4085", version = "2026-10-06T18:04:29+07:00"],],   # IEG
      dependencies = [
          cyjet::dep[id = "3f9a…4a5b", version = "1.2"],
          cyjet::dep[path = "../money"],
      ],
      targets = [cyjet::target[name = "aarch64-macos"],]
  ];
  ```

  Its fields are checked as a struct's are: a typo is "no such field".
  **`name`, `version` and at least one target are required** (owner,
  2026-09-29: "yea"), as in `Synthia.ttsr`. `name` follows the module-name rule, as
  in `Synthia.ttsr` (owner, 2026-09-30: "strict"). A target must be one kayo can
  build, from its one list, `synapse/targets.txt` (owner, 2026-09-30:
  "refuse", as `Synthia.ttsr`); today that is `aarch64-macos` alone, so a
  cross target and its `runner` wait until kayo builds for one. A lone target marked `default = false` means the
  kit has no default: a compile that names no target on the command line
  errors, and one that names it builds (owner, 2026-09-30). A target is
  named with `--target <name>`: `synapse build --target aarch64-macos`
  (owner: "yea"). `version` is any text, at most 256 bytes
  (owner: "same" as `Synthia.ttsr`). **`'prefix'` is the word code writes in
  front of `::`** to reach this kit, picked by its maker and apart from
  its `name` (owner, 2026-09-29: "yea", then b): `prefix = "cash"`. Required (owner: "required"). Two kits with the
  same prefix are an error, fixed by editing one kit's source (owner:
  "Just edit the source code"), so a dependency line names no word:
  `cyjet::dep[path = "../money"]`. Not in the oracle yet. **A
  manifest is data only** (owner, 2026-09-29): anything that must be
  computed lives in **`build.cy`**, a full CyborgPL program (functions,
  loops, a `MAIN`), which **never runs by itself** — a special command runs
  chosen ones or all of them, a dependency's included. Both package managers
  run it. The command is **`kayo prepare`** (owner, 2026-09-29: b):
  `kayo prepare geometry, money` runs those kits' `build.cy` (names
  separated by commas, owner; a space after the comma or not),
  `kayo prepare --all` runs every one; `synthia prepare` alike. Proposed
  alternatives: `build-script`, `run-build`. Compiler claude's four
  suggestions, accepted by the owner: `kayo prepare` acts on the kit in
  the current directory, or a folder given as `synapse run` takes one — a
  word with a `/` is a folder, a bare word a kit's name (owner,
  2026-09-30: "`/`"): `kayo prepare ../app geometry`, the folder first
  as the program is in `synapse run` (owner: "yea"), and a bare `.` or `..`
  is a folder too (owner: "yea"); a name is
  a kit's `name` (owner, 2026-09-30: "I think name's better"; at first
  the local name, then briefly the prefix), and when two dependencies
  share that name, `kayo prepare` refuses and lists them by prefix (owner:
  a), and `kayo prepare --prefix geo2` picks one (owner: "flag"),
  several as a comma list, `--prefix geo2, geo3` (owner: "yea"),
  mixed with names, `kayo prepare money --prefix geo2` (owner: "yea"),
  names first and every word after `--prefix` a prefix (owner: "yea"),
  run in the order written, each once (owner: "yea"); a `build.cy` runs in its own kit's folder; `--all` runs
  dependencies before the kits that use them, each once, stopping at the
  first that fails. With no names it is refused, saying to name kits or use
  `--all` (owner, 2026-09-30: a). A `build.cy` is an ordinary program with
  no limits (owner, 2026-09-30: "It's for fun. You can make a malware
  payload, or be a good person …"): the build has no channel from it, and
  it reaches a kit only as any program would, by reading its files — no
  `my.` or prefix into code (owner: "not a shortcut to reach a limb.
  It's turing complete, so you build that yourself"), and `kayo prepare`
  hands it no words: settings come as any program gets them, from
  environment variables or files (owner: "Might aswell build shit
  yourself"). A
  **`cyjet::dep` is an `id` and `version`** (from the registry) **or a
  `path`** (a folder on this machine) (owner, 2026-09-29: a; its `name`
  was dropped when the prefix became the kit's own):
  `cyjet::dep[path = "../money"]`. The settings the oracle
  knows: `name`, `prefix`, `version`, `dependencies`, `targets` (a list of
  `cyjet::target[name = "aarch64-linux", runner = "qemu-aarch64"]`: several
  targets, since kayo's package manager can do whatever Synthia can, owner
  2026-09-29).

  Synthia stays the separate tool, with `Synthia.ttsr` in its own syntax
  (above). What goes in
  `CyJet.cyr`, and how the two relate, is open.
- **`.limb`** — a *published* kit: the kit's directory (its manifest +
  `.cy` files) packaged as one file for Synthia to upload, download, and
  checksum. Same relationship as a Rust crate folder to its `.crate` tarball.
  Checked 2026-09-14: no existing `.limb` format is registered.

  A kit can never be an opaque precompiled binary: whole-program analysis
  must see inside every kit to prove ranges, borrows, and guardrails across
  the call boundary. A `.limb` therefore always contains analysable source
  (optionally plus a cached front-end result, an optimisation for later).

## The IDE — what it should have (owner, 2026-10-02)

Whether it is its own program or a VS Code extension is open (owner:
"whether it's custom, or VS Code extension"). The owner's list, in the
owner's order:

1. **Quick Fix** — "biggest one": "a way to fix your code instantly (not
   literally) from update breaking your code". When a new CyborgPL
   changes a spelling (CyborgPL is unstable forever, and an old spelling
   is refused with an error naming the new one: `/` → `div`, `<==` →
   `</==`, `let` dropped), Quick Fix rewrites your code to the new
   spelling — Rust's `cargo fix --edition` and Go's `go fix` do the same
   job. Design claude first read it as the usual lightbulb that applies
   a compiler error's `fix:`; the owner corrected that.
   **It is a command too** (owner, 2026-10-02: "a command"), so a whole
   project upgrades without the IDE — **Synthia's** (owner: "I think it
   should be Synthia. Just like how Synthia can already change a strike's
   prefix, by Synthia itself editing the source code"), as its prefix
   override does by hand what a kayo user would (`synthia-manifest.md`).
   Its name is open.

   **How it knows what to rewrite: a spelling table** (owner, 2026-10-02:
   "CyborgPL stores the stable spelling, and maps the stable spelling to
   the current one"). Every language word has a stable ID, and
   **`spellings.cyr`** (one file for the whole language, shipped with
   kayo, read by kayo, Synthia and the IDE) says how each version spells it:
   a row is the ID, the date of the change, the old and the new spelling,
   and the position it applies in. A removal is a spelling of nothing
   (owner: "if `let` was dropped, just make it support that too"), so
   rows cover renames (`/` → `div`), removals (`let`, `pos`) and
   additions (`change` before a bare assignment). kayo's "use X" errors
   and Synthia's rewrites come from the same rows, so they never disagree.
   **It also holds the current keywords and the operators free to declare,
   with the hint shown when one is used undeclared** (owner, 2026-10-13,
   folding in `reserved.cyr`), so the oracle and kayo keep no lists of
   their own. Design claude keeps it, in the same commit as each ruling; a
   test checks it against itself (no hint names a refused word; nothing
   both current and old) and against the naming rule (`syntax-sketch.md`).
   **A meaning change is a row too**, marked to report: Quick Fix lists
   those lines to check instead of rewriting them (`and` running both
   sides, 2026-09-25, is one).

   **Each kit's `cy_ver.cyr`** holds the language date its code was
   written for (owner: "`cpl_ver`", "with the dot"); a file of its own, so
   both manifests and the IDE read it alike. Quick Fix applies every row
   after that date and moves the date forward. It **shows the diff
   first**, and **upgrades an outdated dependency in a copy under
   `.synthia/`**, as the prefix override does. Those parts, and the name
   `spellings.cyr`, are design claude's; the owner: "whatever is good,
   bro". Both files are CyborgPL syntax, so `.cplr`, no new extension.

   **kayo has a command that says where and what to change; Synthia makes
   the changes** (owner, 2026-10-02: "I think Zomp's one should be a
   command, and it tells you where & what to change. Synthia does it
   automatically after a button or a CLI command"). kayo's lists every
   line a row matches — the new spelling, and the lines whose meaning
   changed — and edits nothing; Synthia's applies them, from the IDE's
   button or its own command. Both read `spellings.cyr` and
   `cy_ver.cyr`. Names open.
2. **kit registry search** — searching the index (on `cyjet.dev`) from the
   editor.
3. **Solarized themes** — Solarized Light and Dark.
4. **Highlighting**.
5. **A code picker, like Roblox Studio's**: the autocomplete popup that
   opens while typing — the matching names, an icon for each kind, a
   deprecated one crossed out, and beside the chosen one its signature, a
   one-line description and a link to its documentation:

   ![Roblox Studio's code picker: typing `pr` lists print, printidentity (crossed out), PhysicalProperties, pairs and Path2DControlPoint, with print's signature, description and an Open Documentation link beside them](images/roblox-studio-code-picker.png)

6. **C support** — "though, likely no Quick Fix for C".
7. **LLVM IR support** — no Quick Fix.
8. "and other good stuff".
9. **Two dino games while the prover proves** (owner, 2026-10-03: "we could just
   let the IDE have two. One where the prover "plays", and the other one where
   the writer plays"): Chromium's offline dinosaur game, which is open
   source (Chromium's BSD licence, so it may be bundled with its notice).
   In one, the prover runs: each cactus jumped is a check proven, a refused site
   is a crash, and the run ends with the build. In the other, the writer
   plays while waiting. Design claude's suggestion: a cyborg dino rather
   than Chrome's own art.

Noted by design claude, not ruled: the description in a code picker
needs somewhere to come from, and CyborgPL has no doc comments yet; in VS
Code, 3 ships built in, 6 is the existing `clangd` extension, 7 has
existing `.ll` extensions, and 1, 2 and 5 come from a language server
running kayo.

## Cross-compiling, running elsewhere (discussed 2026-09-21, proposal)

kayo compiles and stops; it never executes what it built. The target is
declared in the manifest (`Synthia.ttsr` or `CyJet.cyr`), never sniffed from the host; the host is
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

- Synthia's own syntax for `Synthia.ttsr`, and how Synthia and kayo's package manager relate.
- **The kit registry is an index only** (owner, 2026-09-29): small files naming each kit's versions and linking to each version's release where its author keeps it; the registry never stores code, so it costs nothing to host (static files). A release lives **on GitHub**; the index keeps **a checksum of every release**; **registering a new kit is a reviewed pull request, and a new version needs no review**; and **malware-free is not guaranteed**, which the registry says plainly. Open: how an update proves it comes from the kit's owner. Its home is **`cyjet.dev`** (owner, 2026-10-17: ".dev is main"; cyjet.org leads to it), which is also CyJet's website (owner, 2026-10-16: "`cyjet.org` serves both as the PM's website, and the registry"). Chosen before, when a library was a limb: `limbs.xxx` (owner, 2026-09-29: "`limbs.xxx` it is, I'll just wait until there's a limbs registery"; never bought).
