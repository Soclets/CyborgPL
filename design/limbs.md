# Neurons — reaching into one

> Revised 2026-10-07: code here is in the new syntax; an old spelling in the prose is history. The old → new table is `revision-2026-10-07.md`.

Decided 2026-09-19 onward. The module-system pieces settled so far; what
is still open is listed at the end.

## No `import`, no `use`

- **Dependencies are declared in the manifest** (`Synthia.ttsr`, or `Kinesin.nrp` for CCA's built-in package manager, 2026-09-29), never in source. A source
  file has no statement that loads a Neuron.
- **Every item is reached by its full path**: `std::print.useful.stdout[…]`,
  `mathstuff::pow[2, 64]`, `'r': std::uenr`, `crate_x::Locker[…]`. `::`
  crosses the boundary and the reader always sees where a thing comes from
  (decided 2026-09-14). This is Rust's model (Cargo lists, paths reach), not
  Python's (`import` loads and binds).
- **There is no user-side alias.** Nothing like Rust's
  `use mathstuff::pow;` exists for the *user* of a Neuron.

## No bare reach from another Neuron (owner, 2026-09-23)

Everything from another Neuron is written with its path, always:
`mathstuff::pow[2, 64]`, `crate_x::Locker[…]`, `std::print.useful.stdout[…]`.
No Neuron can offer a shortcut, so a bare name like `pow[…]` always
means something in your own module, and two Neurons can never clash over
one.

Operators are the one thing that comes bare, because there is nowhere
to put a path in `'u' dot 'v'`. An operator is written for its operand
types, so which `dot` is meant follows from what `'u'` is. Two meanings
of one operator for the same types is a compile error (owner,
2026-09-23; `operators.md`, "From another Limb").

History: on 2026-09-19 the owner made bare reach the Neuron author's
choice, stated in the Neuron's source. Earlier on 2026-09-23 its spelling
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
`my.to[…]` inside its own Neuron (`limb.to[…]` until 2026-10-06; owner floated `std:to` / `std:::to` for
the inside; `:` already attaches types and `limb.` already means "my own
Neuron," so `limb.` it is). Bare `to[…]` is the keyword. Structs, consts and
modules keep the strict rule — their names appear bare in type positions.

## `pub` and `const` (decided 2026-09-19)

```
pub fn add ['a': int32, 'b': int32] -> int32 { … }
pub struct Locker { … }
pub const 'max_players' := 8;
const 'pi': bin64 = 3.141_592_653_589_793;      # private to the Limb
```

- **`pub`** is a bare word before `fn`, `struct`, or `const` at the top
  level: visible to other Neurons. Without it, private to this Neuron. `MAIN`
  is never `pub`.
- **A Neuron's top-level value is a `const`**, not a variable (a variable
  is refused at the top level). It is a value known at compile time — a
  literal, arithmetic on literals, another const, a pure call; anything
  with an effect or a runtime input is a compile error. It has no storage:
  Cortex bakes it in at every use. It is never assignable, never `mut` /
  `!mut`, never shadowed, and no ownership verb applies to it — nobody
  owns it, every use is a fresh value.
- **Named like a variable, quoted**: `'pi'`, used in expressions as
  `'pi' times 2`, reached from another Neuron as `mathstuff::'pi'` (owner,
  2026-09-19: "single quoted"). One rule — quotes mean a user-named value.
- **The OS door is open to every Neuron** (2026-09-19): `extern`, `ptr[T]`
  and inline asm are base-language features (`ffi.md`); std is the
  library everyone uses for them, not a privileged one.
- **There is no public variable** (owner, 2026-09-19: "No public
  variable"). A Neuron exposes constants, functions, and structs; state
  another Neuron may read *and assign* does not exist. The uses a public
  variable serves have owned shapes: a configuration knob is a parameter or
  a struct field; a shared counter or cache is a value someone owns and
  lends mutably; a singleton is what `MAIN` owns and hands down. A global
  has no owner, so ownership, whole-program analysis, and automatic
  parallelism would all have a hole in it. Closed, not parked.
- This does not reopen "no `const` keyword for locals" (2026-09-14): inside
  a body a variable declaration (`let 'x' := …`) is still the only kind and
  Cortex finds the constants itself. `const` exists only where variables
  do not.

## Modules (decided 2026-09-19)

A Neuron has modules: named boxes inside it, each with its own namespace and
its own private/public line. Owner: "we should have modules"; then (c) of
file-only / block-only / both.

### Where a module comes from

- **Every `.nrv` file in the Neuron is a module named after the file**:
  `shapes.nrv` → `shapes`; a directory nests: `shapes/circle.nrv` →
  `shapes.circle`. Nothing declares it — Synthia knows the files, and `ls`
  shows the tree.
- **`module name { … }` inside a file** nests one more module that has no
  file of its own — for a private helper namespace or tests beside the code.
  The keyword is `module`, not `mod`: `mod` is the remainder operator.
  There is no `module name;` include line (Rust's `mod x;` exists only
  because Cargo does not scan).
- A module cannot be both a file and a block.
- **The root is `neuron.nrv`** (kept over `root.nrv`; the echo with the
  `.limb` archive is fine — that is the whole Neuron packed, this is the file
  inside it). Its items are the Neuron's top level, `geometry::…` from outside.
  `MAIN` lives there and nowhere else. A small Neuron is one `neuron.nrv`.

### Names — "whatever has most compat"

A module name, file or block, is a lowercase ASCII identifier: `a`–`z`,
`0`–`9`, `_`, starting with a letter. Not a keyword; not a Windows device
name (`con`, `nul`, `aux`, `prn`, `com1`–`9`, `lpt1`–`9`). A file in the
Neuron whose name does not fit is a compile error naming it, never silently
skipped. Why: macOS and Windows file systems are case-insensitive
(`Shapes.nrv` and `shapes.nrv` collide on some machines and not others),
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
  fallback and over the Neuron's own name — a root item added later cannot
  break a distant module, and a Neuron's source does not depend on its
  package name). Rust's `crate::`.
- From outside, the Neuron's prefix replaces `my`: `geometry::shapes.area`
  — the same path with the boundary marked.

### Visibility — three audiences

```
fn helper [] { … }                 # this module and its descendants
pub.neuron fn shared [] { … }        # the whole Limb, not outside
pub fn api [] { … }                # other Limbs too
pub.neuron module tests { … }        # modules carry it as well
```

- Nothing: private to the module it is declared in, visible to that module
  and the modules nested inside it (so a block reaches its file's private
  helpers, and `my.helper` works from anywhere when `helper` is at the
  root).
- **`pub.neuron`**: the rest of this Neuron. Owner chose the chain over Rust's
  `pub(crate)` — `( )` is arithmetic grouping only — and over having no
  middle level. `pub.neuron` narrows `pub` the way `lend.mut` narrows `lend`.
- **`pub`**: other Neurons.
- Every module on a path is checked the same way: a `pub` item inside a
  private module is unreachable from outside that module's parent.
- A file module is public (provisional: a file has no declaration line to
  carry `pub.neuron`; each item inside still carries its own visibility).

## One name, one thing (decided 2026-09-21)

A module and a type (struct, enum, interface) cannot share a name in a
Neuron: `module shapes { }` beside `struct shapes { }` — or a file
`shapes.nrv` beside it — is a compile error, "rename one". Proposed
alternative: allow it and tell them apart by what follows the dot
(`shapes.'pi'` the module, `shapes.claimed` the type). Owner chose the
refusal: nothing to disambiguate, for the reader or for Cortex.

## A shorter name for users — `pub fn area = shapes.circle.area;` (owner, 2026-09-23)

```
# geometry/shapes/circle.nrv
pub.neuron fn area ['r': bin64] -> bin64 { … }
pub.neuron struct Circle { r: bin64 }

# geometry/neuron.nrv
pub fn area = shapes.circle.area;          # users write geometry::area[…]
pub struct Circle = shapes.circle.Circle;  # users write geometry::Circle[…]
```

Read it as: other Neurons get a function called `area`, and it is the one
written in `shapes/circle.nrv`. The Neuron's author decides what comes
after `geometry::`, whatever the folders look like inside (owner:
"Doesn't the Limb maker say what goes after `::`?"). The line reads as
a declaration: the kind, the name users get, and `=` the path to where
it is written. That path is the ordinary one inside a Neuron, relative to
the module the line is in (2026-09-19): `shapes.circle` is how the file
`shapes/circle.nrv` is named everywhere, and source never writes a
file name, since a module may also be a `module` block with no file.
The line adds a name only: no code is copied, and it is the same
`area`. If the original is `pub.neuron`, only the short name exists
outside, so the folder layout stays private. If it is `pub`, both
paths work.

Proposed alternatives: none, where the path follows the folders and an
author wanting `geometry::area` puts it in `neuron.nrv` or writes a
forwarding function (design claude's first recommendation, for one name
per thing); `pub shapes.circle.area;`, refused by the owner ("I don't
think that makes sense, the syntax") because the new name is only
implied; `pub fn area from shapes.circle;`, English-like but unable to
give a different name; a file path, `"shapes/circle.nrv".area`, after
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
# std/str.nrv
pub fn split = split.split;      # users write std::str.split[…]
```

So a big Neuron keeps its grouping (`std::str.…`) while the code behind
it lives in whatever files suit it. Proposed alternative: only in
`neuron.nrv`, so a Neuron's whole outside face is in one file; but then
every short name would sit one level deep, and keeping
`std::str.split` would mean the real function has to live in `str.nrv`.

Not yet ruled:
which kinds it covers beyond `fn` and `struct` (`enum`, `interface`,
`const`, `module`).

## A Neuron's name (owner, 2026-09-23: a)

The name lives in **the manifest** (`Synthia.ttsr` / `Kinesin.nrp`), beside the version and the
dependencies, and nowhere in source. Synthia reads it and hands it to
CCA with the rest of what it is building. Renaming the directory changes
nothing, and a Neuron's own code never names itself: it says `my.` (`limb.` until 2026-10-06).
Proposed alternative: `limb mathstuff;` at the top of `neuron.nrv`, which
would give the name two places to disagree.

**A dependency may have a MAIN** (owner, 2026-09-29: a): it runs only when
that Neuron itself is run, and is ignored when another Neuron uses it, so a
library can ship its own demo or tool.

## Case (owner, 2026-10-06)

Names are exact (`case.sensitive`) unless a Neuron's writer marks them
`case.insensitive`: on kinds in `neuron.nrv` (`case.insensitive [fn,
struct, enum, interface, const, case, field, var];`), on one item (`pub
case.insensitive fn area …`), on a block module, or as a file's first
line (`case.insensitive;`). Writing `case.sensitive` is allowed; marks
that disagree are an error. An insensitive name matches in any case, by
Unicode's full case folding, in its own Neuron and from other Neurons.
Keywords stay exact. It costs nothing at run time. DECISIONS 2026-10-06
has the rulings and the oracle's readings; `examples/case_marks.nrv`,
`examples/uses_case`.

## Open

- A file module's own visibility (provisionally public).
