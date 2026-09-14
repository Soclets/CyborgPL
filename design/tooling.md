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

## Open

- Whether `JetPack.toml` is also the name of the CLI (`jetpack add …`).
