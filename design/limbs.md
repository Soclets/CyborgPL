# Limbs — reaching into one

Decided 2026-09-19 onward. The module-system pieces settled so far; what
is still open is listed at the end.

## No `import`, no `use`

- **Dependencies are declared in the manifest** (`Synthia.ttsr`, or `CCA.cplr` for CCA's built-in package manager, 2026-09-29), never in source. A source
  file has no statement that loads a Limb.
- **Every item is reached by its full path**: `std::print.useful.stdout[…]`,
  `mathstuff::pow[2, 64]`, `'r': std::uenr`, `crate_x::Locker[…]`. `::`
  crosses the boundary and the reader always sees where a thing comes from
  (decided 2026-09-14). This is Rust's model (Cargo lists, paths reach), not
  Python's (`import` loads and binds).
- **There is no user-side alias.** Nothing like Rust's
  `use mathstuff::pow;` exists for the *user* of a Limb.

## No bare reach from another Limb (owner, 2026-09-23)

Everything from another Limb is written with its path, always:
`mathstuff::pow[2, 64]`, `crate_x::Locker[…]`, `std::print.useful.stdout[…]`.
No Limb can offer a shortcut, so a bare name like `pow[…]` always
means something in your own module, and two Limbs can never clash over
one.

Operators are the one thing that comes bare, because there is nowhere
to put a path in `'u' dot 'v'`. An operator is written for its operand
types, so which `dot` is meant follows from what `'u'` is. Two meanings
of one operator for the same types is a compile error (owner,
2026-09-23; `operators.md`, "From another Limb").

History: on 2026-09-19 the owner made bare reach the Limb author's
choice, stated in the Limb's source. Earlier on 2026-09-23 its spelling
was chosen as `pub bare fn pow […]`, a separate word rather than
`pub.bare`, since bare changes how a name is written and not who sees
it. The owner then asked why anything but operators would need to be
bare. Operators already cover two-operand calls, and bare would have
been the only break in "the reader always sees where a thing comes
from" (2026-09-14). Owner: "`mathstuff::pow` is fine", then "drop
bare".

## A keyword may name a function — reached by path, never bare (decided 2026-09-19)

`::` already says "what follows is a name in that Limb," so `std::to[…]`
is unambiguous even though `to` is a keyword; the reservation rule only
ever protected *bare* positions. So `fn to […]` is legal, and such a
function is reached through a path everywhere: `std::to[…]` from outside,
`my.to[…]` inside its own Limb (`limb.to[…]` until 2026-10-06; owner floated `std:to` / `std:::to` for
the inside; `:` already attaches types and `limb.` already means "my own
Limb," so `limb.` it is). Bare `to[…]` is the keyword. Structs, consts and
modules keep the strict rule — their names appear bare in type positions.

## `pub` and `const` (decided 2026-09-19)

```
pub fn add ['a': int32, 'b': int32] --> int32 { … }
pub struct Locker { … }
pub const 'max_players' := 8;
const 'pi': bin64 = 3.141_592_653_589_793;      # private to the Limb
```

- **`pub`** is a bare word before `fn`, `struct`, or `const` at the top
  level: visible to other Limbs. Without it, private to this Limb. `MAIN`
  is never `pub`.
- **A Limb's top-level value is a `const`**, not a variable (a variable
  is refused at the top level). It is a value known at compile time — a
  literal, arithmetic on literals, another const, a pure call; anything
  with an effect or a runtime input is a compile error. It has no storage:
  Precog bakes it in at every use. It is never assignable, never `mut` /
  `!mut`, never shadowed, and no ownership verb applies to it — nobody
  owns it, every use is a fresh value.
- **Named like a variable, quoted**: `'pi'`, used in expressions as
  `'pi' times 2`, reached from another Limb as `mathstuff::'pi'` (owner,
  2026-09-19: "single quoted"). One rule — quotes mean a user-named value.
- **The OS door is open to every Limb** (2026-09-19): `extern`, `ptr[T]`
  and inline asm are base-language features (`ffi.md`); std is the
  library everyone uses for them, not a privileged one.
- **There is no public variable** (owner, 2026-09-19: "No public
  variable"). A Limb exposes constants, functions, and structs; state
  another Limb may read *and assign* does not exist. The uses a public
  variable serves have owned shapes: a configuration knob is a parameter or
  a struct field; a shared counter or cache is a value someone owns and
  lends mutably; a singleton is what `MAIN` owns and hands down. A global
  has no owner, so ownership, whole-program analysis, and automatic
  parallelism would all have a hole in it. Closed, not parked.
- This does not reopen "no `const` keyword for locals" (2026-09-14): inside
  a body a variable declaration (`let 'x' := …`) is still the only kind and
  Precog finds the constants itself. `const` exists only where variables
  do not.

## Modules (decided 2026-09-19)

A Limb has modules: named boxes inside it, each with its own namespace and
its own private/public line. Owner: "we should have modules"; then (c) of
file-only / block-only / both.

### Where a module comes from

- **Every `.cpls` file in the Limb is a module named after the file**:
  `shapes.cpls` → `shapes`; a directory nests: `shapes/circle.cpls` →
  `shapes.circle`. Nothing declares it — Synthia knows the files, and `ls`
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

### Reaching in — `::` crosses, `.` narrows, `my.` is the root

```
shapes.area[…]                 # from the root: a module's item
my.helper[…]                   # from inside a module: the root's item
my.shapes.circle.area[…]       # from inside any module: an absolute path
circle.area[…]                 # from inside `shapes`: its own child
geometry::shapes.circle.area[…]   # from another Limb
'r': shapes.circle.Circle      # a struct through modules
shapes.circle.'pi'             # a const through modules
```

- A bare name is **this module's**. A dotted path is **relative to this
  module**. `my.` starts at the root (`limb.` until 2026-10-06, when the
  owner chose `my.`; owner chose (c) over lexical
  fallback and over the Limb's own name — a root item added later cannot
  break a distant module, and a Limb's source does not depend on its
  package name). Rust's `crate::`.
- From outside, the Limb's prefix replaces `my`: `geometry::shapes.area`
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
  helpers, and `my.helper` works from anywhere when `helper` is at the
  root).
- **`pub.limb`**: the rest of this Limb. Owner chose the chain over Rust's
  `pub(crate)` — `( )` is arithmetic grouping only — and over having no
  middle level. `pub.limb` narrows `pub` the way `lend.mut` narrows `lend`.
- **`pub`**: other Limbs.
- Every module on a path is checked the same way: a `pub` item inside a
  private module is unreachable from outside that module's parent.
- A file module is public (provisional: a file has no declaration line to
  carry `pub.limb`; each item inside still carries its own visibility).

## One name, one thing (decided 2026-09-21)

A module and a type (struct, enum, interface) cannot share a name in a
Limb: `module shapes { }` beside `struct shapes { }` — or a file
`shapes.cpls` beside it — is a compile error, "rename one". Proposed
alternative: allow it and tell them apart by what follows the dot
(`shapes.'pi'` the module, `shapes.claimed` the type). Owner chose the
refusal: nothing to disambiguate, for the reader or for Precog.

## A shorter name for users — `pub fn area = shapes.circle.area;` (owner, 2026-09-23)

```
# geometry/shapes/circle.cpls
pub.limb fn area ['r': bin64] --> bin64 { … }
pub.limb struct Circle { r: bin64 }

# geometry/limb.cpls
pub fn area = shapes.circle.area;          # users write geometry::area[…]
pub struct Circle = shapes.circle.Circle;  # users write geometry::Circle[…]
```

Read it as: other Limbs get a function called `area`, and it is the one
written in `shapes/circle.cpls`. The Limb's author decides what comes
after `geometry::`, whatever the folders look like inside (owner:
"Doesn't the Limb maker say what goes after `::`?"). The line reads as
a declaration: the kind, the name users get, and `=` the path to where
it is written. That path is the ordinary one inside a Limb, relative to
the module the line is in (2026-09-19): `shapes.circle` is how the file
`shapes/circle.cpls` is named everywhere, and source never writes a
file name, since a module may also be a `module` block with no file.
The line adds a name only: no code is copied, and it is the same
`area`. If the original is `pub.limb`, only the short name exists
outside, so the folder layout stays private. If it is `pub`, both
paths work.

Proposed alternatives: none, where the path follows the folders and an
author wanting `geometry::area` puts it in `limb.cpls` or writes a
forwarding function (design claude's first recommendation, for one name
per thing); `pub shapes.circle.area;`, refused by the owner ("I don't
think that makes sense, the syntax") because the new name is only
implied; `pub fn area from shapes.circle;`, English-like but unable to
give a different name; a file path, `"shapes/circle.cpls".area`, after
the owner said the module path "doesn't seem like it's referencing a
file". It would be the only file name in source, and it could not
reach a `module` block.

**The name may differ from the original's** (owner, 2026-09-23: a). The
name before `=` is whatever users should type:

```
pub fn new_circle = shapes.circle.new;
pub fn new_square = shapes.square.new;
```

Two files that each have a `new` can both be offered at the root. The
line spells out both names, so searching for `new_circle` finds where
it really is. Proposed alternative: the name must match the original,
so the author renames inside their own files first.

**It may stand in any module** (owner, 2026-09-23: "in any file"), and
adds the name to that module:

```
# std/str.cpls
pub fn split = split.split;      # users write std::str.split[…]
```

So a big Limb keeps its grouping (`std::str.…`) while the code behind
it lives in whatever files suit it. Proposed alternative: only in
`limb.cpls`, so a Limb's whole outside face is in one file; but then
every short name would sit one level deep, and keeping
`std::str.split` would mean the real function has to live in `str.cpls`.

Not yet ruled:
which kinds it covers beyond `fn` and `struct` (`enum`, `interface`,
`const`, `module`).

## A Limb's name (owner, 2026-09-23: a)

The name lives in **the manifest** (`Synthia.ttsr` / `CCA.cplr`), beside the version and the
dependencies, and nowhere in source. Synthia reads it and hands it to
CCA with the rest of what it is building. Renaming the directory changes
nothing, and a Limb's own code never names itself: it says `my.` (`limb.` until 2026-10-06).
Proposed alternative: `limb mathstuff;` at the top of `limb.cpls`, which
would give the name two places to disagree.

**A dependency may have a MAIN** (owner, 2026-09-29: a): it runs only when
that Limb itself is run, and is ignored when another Limb uses it, so a
library can ship its own demo or tool.

## Open

- A file module's own visibility (provisionally public).
