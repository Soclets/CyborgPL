# Tooling names

Owner, 2026-09-14.

- **Cyborg Chassis Assembler (CCA)** — the CyborgPL compiler (owner,
  2026-09-18; spelling of "Chassis" to confirm — owner wrote "Chasis").
  Not an assembler in the `as` sense; the owner accepts the possible
  confusion. The chassis is the frame; Precog is what is bolted to it.
- **Nowork** — the N% compiler (owner, 2026-09-18): the iteration compiler
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
  every library or program is a Limb, JetPack installs Limbs, and the standard
  library is one Limb among them, reached as `std.` in a chain. (A cyborg has
  limbs.)
- **JetPack** — the package manager. *Jet* = fast, *Pack* = package: a fast
  package. Its manifest file is `JetPack.toml`.

- **`.cpls`** — a CyborgPL source file (*CyborgPL Source*). `.cpl` is
  deliberately avoided.
- **`.limb`** — a *published* Limb: the Limb's directory (`JetPack.toml` +
  `.cpls` files) packaged as one file for JetPack to upload, download, and
  checksum. Same relationship as a Rust crate folder to its `.crate` tarball.
  Checked 2026-09-14: no existing `.limb` format is registered.

  A Limb can never be an opaque precompiled binary: whole-program analysis
  must see inside every Limb to prove ranges, borrows, and guardrails across
  the call boundary. A `.limb` therefore always contains analysable source
  (optionally plus a cached front-end result, an optimisation for later).

## Open

- Whether `JetPack.toml` is also the name of the CLI (`jetpack add …`).
