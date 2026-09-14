# Tooling names

Owner, 2026-09-14.

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
