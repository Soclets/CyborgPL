# `CyJet.toml`, the manifest, and `CyJet.lock` (owner, 2026-10-22)

**`CyJet.toml` replaces `CyJet.cyr`** (owner: "I'm feeling like changing CyJet to `CyJet.toml`, but it's Tankun instead
of Tom"). The `.toml` is **Tankun's Obvious Minimal Language** (owner: "yea"): the owner's own format with TOML's
initials, not TOML (owner: "(b)", over real TOML), and a new syntax (owner: "(2)", over today's `cyjet::manifest[…]`
value renamed, and "I wanna design a new one", over Synthia's `.ttsr` style).

```
my details:
name = shop
prefix = shop
ver = 1.0.0

my kits:
std {
id = fff4302dffc4a894a2bab6baf724eb21dfc05a2ad9818509d8001013d6761936
ver = 2026-10-06T12:57:14+07:00
}
geometry {
path = ../geometry
}

my limbs:
cnc {
id = 04b99a7cbb141b84b9f2f38c085c124e2badc37d663f71c6deebdd6ac24e79b1
ver = 2026-10-10T16:42:00+07:00
}

my targets:
aarch64-macos {
}
thumbv7em-none {
table_bytes = 4096
}
```

- **A section is a phrase ending in `:`**; under it come `key = value` lines, with no indentation and no quotes (the
  owner's own sketch). The sections are **`my details:`**, **`my kits:`**, **`my limbs:`** and **`my targets:`**
  (owner: "`my targets`", "(a)" to limbs having their own section, "so `my kits` too?"); `kit:` / `needs:` /
  `targets:` were picked first and replaced by the owner's sketch.
- **A kit, limb or target is a block**, its name then `{ … }` (owner's sketch): a registry kit has `id` and `ver`, a
  folder kit `path` (owner: "(a)", over `folder`), std from a folder `std { path = ../std }`. A target holds its own
  settings (`table_bytes`, 2026-10-19), an empty block when it has none (owner: "(a)", over dotted lines).
- **`ver`, never `version`**, in every section (owner: "(a)").
- **Comments are CyborgPL's family**: `#` to the end of the line, `#wl`, `#Nl`, `#Nld` (owner: "`#` + the other
  ways, like `#3l`").

## `CyJet.lock`

**A lockfile** (owner: "(a)"): the manifest lists only what a kit uses directly; CyJet works out the dependencies of
dependencies (Money needing `d64`, say), picks a version of each, and writes every kit and limb with its exact
version and a checksum into `CyJet.lock`, which builds read until an update rewrites it. **It is real binary**,
packed bytes (owner: "But it's funny, and you get technically more speed", after "Would you really read a
lockfile?"), so git neither diffs nor merges it (a conflict is fixed by regenerating), and a command shows it,
`cyjet show lock`, with each indirect dependency's "needed by" (owner: "Shouldn't `from` be a comment?", when it was
still text).

## Open

- The lockfile's byte layout and its format version; the command names (`cyjet update`, `cyjet show lock`, `cyjet
  why`) are design claude's sketches, CyJet having no command of its own yet (2026-10-15).
- Version conflicts: two kits wanting different versions of one dependency (one version picked, or both side by side).
- Whether a `CyJet.toml` may be concise (2026-10-22 allowed a concise `.cyr` manifest; this file is no longer
  CyborgPL).
- Not built: the oracle and kayo still read `CyJet.cyr` (16 examples, std/, the reject tests).
