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
- **There is no public variable** (owner, 2026-09-19: "No public
  variable"). A Limb exposes constants, functions, and structs; state
  another Limb may read *and assign* does not exist. The uses a public
  variable serves have owned shapes: a configuration knob is a parameter or
  a struct field; a shared counter or cache is a value someone owns and
  lends mutably; a singleton is what `MAIN` owns and hands down. A global
  has no owner, so ownership, whole-program analysis, and automatic
  parallelism would all have a hole in it. Closed, not parked.
- This does not reopen "no `const` keyword for locals" (2026-09-14): inside
  a body `let` is still the only declaration and Precog finds the constants
  itself. `const` exists only where `let` does not.

## Modules (decided 2026-09-19)

A Limb has modules: named boxes inside it, each with its own namespace and
its own private/public line. Owner: "we should have modules"; then (c) of
file-only / block-only / both.

### Where a module comes from

- **Every `.cpls` file in the Limb is a module named after the file**:
  `shapes.cpls` → `shapes`; a directory nests: `shapes/circle.cpls` →
  `shapes.circle`. Nothing declares it — JetPack knows the files, and `ls`
  shows the tree.
- **`module name { … }` inside a file** nests one more module that has no
  file of its own — for a private helper namespace or tests beside the code.
  The keyword is `module`, not `mod`: `mod` is the remainder operator.
  There is no `module name;` include line (Rust's `mod x;` exists only
  because Cargo does not scan).
- A module cannot be both a file and a block.
- **The root is `limb.cpls`** (kept over `root.cpls`; the echo with the
  `.limb` archive is fine — that is the whole Limb packed, this is the file
  inside it). Its items are the Limb's top level, `geometry::…` from outside.
  `MAIN` lives there and nowhere else. A small Limb is one `limb.cpls`.

### Names — "whatever has most compat"

A module name, file or block, is a lowercase ASCII identifier: `a`–`z`,
`0`–`9`, `_`, starting with a letter. Not a keyword; not a Windows device
name (`con`, `nul`, `aux`, `prn`, `com1`–`9`, `lpt1`–`9`). A file in the
Limb whose name does not fit is a compile error naming it, never silently
skipped. Why: macOS and Windows file systems are case-insensitive
(`Shapes.cpls` and `shapes.cpls` collide on some machines and not others),
and macOS normalises Unicode names differently from Linux.

### Reaching in — `::` crosses, `.` narrows, `limb.` is the root

```
shapes.area[…]                 # from the root: a module's item
limb.helper[…]                 # from inside a module: the root's item
limb.shapes.circle.area[…]     # from inside any module: an absolute path
circle.area[…]                 # from inside `shapes`: its own child
geometry::shapes.circle.area[…]   # from another Limb
'r': shapes.circle.Circle      # a struct through modules
shapes.circle.'pi'             # a const through modules
```

- A bare name is **this module's**. A dotted path is **relative to this
  module**. `limb.` starts at the root (owner chose (c) over lexical
  fallback and over the Limb's own name — a root item added later cannot
  break a distant module, and a Limb's source does not depend on its
  package name). Rust's `crate::`, spelled with the word that already names
  the root file.
- From outside, the Limb's name replaces `limb`: `geometry::shapes.area`
  — the same path with the boundary marked.

### Visibility — three audiences

```
fn helper [] { … }                 # this module and its descendants
pub.limb fn shared [] { … }        # the whole Limb, not outside
pub fn api [] { … }                # other Limbs too
pub.limb module tests { … }        # modules carry it as well
```

- Nothing: private to the module it is declared in, visible to that module
  and the modules nested inside it (so a block reaches its file's private
  helpers, and `limb.helper` works from anywhere when `helper` is at the
  root).
- **`pub.limb`**: the rest of this Limb. Owner chose the chain over Rust's
  `pub(crate)` — `( )` is arithmetic grouping only — and over having no
  middle level. `pub.limb` narrows `pub` the way `lend.mut` narrows `lend`.
- **`pub`**: other Limbs.
- Every module on a path is checked the same way: a `pub` item inside a
  private module is unreachable from outside that module's parent.
- A file module is public (provisional: a file has no declaration line to
  carry `pub.limb`; each item inside still carries its own visibility).

## Open

- The spelling of the author's "reachable bare" / re-export choice — a
  flag beside or on `pub`; with modules it is also how `shapes.circle.area`
  becomes `geometry::area` for users. Undecided.
- A file module's own visibility (provisionally public).
- Limb name declared in source, or only in `JetPack.toml`? (Claude,
  2026-09-14.)
