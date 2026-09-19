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

## `pub` and `const` (decided 2026-09-19)

```
pub fn add ['a': int32, 'b': int32] -> int32 { … }
pub struct Locker { … }
pub const 'max_players' := 8;
const 'pi': bin64 = 3.141_592_653_589_793;      # private to the Limb
```

- **`pub`** is a bare word before `fn`, `struct`, or `const` at the top
  level: visible to other Limbs. Without it, private to this Limb. `MAIN`
  is never `pub`.
- **A Limb's top-level value is a `const`**, not a `let` (`let` stays
  refused at the top level). It is a value known at compile time — a
  literal, arithmetic on literals, another const, a pure call; anything
  with an effect or a runtime input is a compile error. It has no storage:
  Precog bakes it in at every use. It is never assignable, never `mut` /
  `immut`, never shadowed, and no ownership verb applies to it — nobody
  owns it, every use is a fresh value.
- **Named like a variable, quoted**: `'pi'`, used in expressions as
  `'pi' x 2`, reached from another Limb as `mathstuff::'pi'` (owner,
  2026-09-19: "single quoted"). One rule — quotes mean a user-named value.
- Why not `let` at the top level: the only honest use of a global is a
  named constant a Limb shares. Configuration read once is a boundary
  value and arrives as a parameter; genuinely mutable program-wide state
  has no owner and conflicts with the ownership model (Rust makes
  `static mut` unsafe for the same reason) — if ever wanted, it is a
  concurrency-era design, not a `let`.
- This does not reopen "no `const` keyword for locals" (2026-09-14): inside
  a body `let` is still the only declaration and Precog finds the constants
  itself. `const` exists only where `let` does not.

## Open

- The spelling of the author's "reachable bare" choice — a flag beside or
  on `pub`. Undecided.
- Limb name declared in source, or only in `JetPack.toml`? (Claude,
  2026-09-14.)
