# Limbs — reaching into one

Decided 2026-09-19. The module-system pieces settled so far; the rest
(`pub`, the spelling of the author's choice) is open.

## No `import`, no `use`

- **Dependencies are declared in `JetPack.toml`**, never in source. A source
  file has no statement that loads a Limb.
- **Every item is reached by its full path**: `std::printu.stdout[…]`,
  `mathstuff::pow[2, 64]`, `'r': std::uer`, `crate_x::Locker[…]`. `::`
  crosses the boundary and the reader always sees where a thing comes from
  (decided 2026-09-14). This is Rust's model (Cargo lists, paths reach), not
  Python's (`import` loads and binds).
- **There is no user-side alias.** Nothing like Rust's
  `use mathstuff::pow;` exists for the *user* of a Limb.

## The Limb maker decides (owner, 2026-09-19)

Whether an item may be reached **bare** — `pow[2, 64]` with no `mathstuff::`
— is the Limb author's call, stated in the Limb's own source, per item or
per Limb. A user cannot override it from their side. "If you want to
override that, you edit the source code yourself" — which is always
possible: a `.limb` is analysable source, and the licence allows it. Forking
the Limb is the override.

Consequences:
- Two Limbs that both export a bare `pow` collide in the user's program;
  that is a compile error naming both, and the user's fix is to drop one
  Limb, fork one, or (if the author allowed both forms) write the full path
  for one of them.
- The std Limb's items are reached `std::…` unless std's own source says
  otherwise. Whether std exports anything bare is std's decision, made
  when std is written.

## Open

- The spelling of the author's choice. It lands on `pub` (raised
  2026-09-19): the same declaration that says "visible outside the Limb"
  can say "and reachable bare". Undecided.
- Limb name declared in source, or only in `JetPack.toml`? (Claude,
  2026-09-14.)
