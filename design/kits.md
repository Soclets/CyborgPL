# kits — reaching into one

> Revised 2026-10-07: code here is in the new syntax; an old spelling in the prose is history. The old → new table is `revision-2026-10-07.md`.

Decided 2026-09-19 onward. The module-system pieces settled so far; what
is still open is listed at the end.

## No `import`, no `use`

- **Dependencies are declared in the manifest** (`Synthia.ttsr`, or `CyJet.toml` for kayo's built-in package manager, `CyJet.cyr` from 2026-09-29 to 2026-10-22), never in source. A source
  file has no statement that loads a kit.
- **Every item is reached by its full path**: `std::print.useful.stdout[…]`,
  `mathstuff::pow[2, 64]`, `'r': std::uenr`, `crate_x::Locker[…]`. `::`
  crosses the boundary and the reader always sees where a thing comes from
  (decided 2026-09-14). This is Rust's model (Cargo lists, paths reach), not
  Python's (`import` loads and binds).
- **There is no user-side alias.** Nothing like Rust's
  `use mathstuff::pow;` exists for the *user* of a kit.

## No bare reach from another kit (owner, 2026-09-23)

Everything from another kit is written with its path, always:
`mathstuff::pow[2, 64]`, `crate_x::Locker[…]`, `std::print.useful.stdout[…]`.
No kit can offer a shortcut, so a bare name like `pow[…]` always
means something in your own module, and two kits can never clash over
one.

Operators are the one thing that comes bare, because there is nowhere
to put a path in `'u' dot 'v'`. An operator is written for its operand
types, so which `dot` is meant follows from what `'u'` is. Two meanings
of one operator for the same types is a compile error (owner,
2026-09-23; `operators.md`, "From another Limb").

History: on 2026-09-19 the owner made bare reach the kit author's
choice, stated in the kit's source. Earlier on 2026-09-23 its spelling
was chosen as `pub bare fn pow […]`, a separate word rather than
`pub.bare`, since bare changes how a name is written and not who sees
it. The owner then asked why anything but operators would need to be
bare. Operators already cover two-operand calls, and bare would have
been the only break in "the reader always sees where a thing comes
from" (2026-09-14). Owner: "`mathstuff::pow` is fine", then "drop
bare".

## Another kit's names take its path; `use` renames one for a file (owner, 2026-10-22)

**Another kit's names always take its path** (owner: "What IF there's no bare.optional? It's always not bare.", "Only
other kits, own stays bare."): its functions, types, enums and enum values, std's included.

```
std::print.shortest(greet::hello(), newln);
var $xs: std::ls[int.sign.64] = [];
std::ls.append(lend.mut $xs, value: 3);
let $b: greet::Note := greet::Note[text = "hi"];
let $up := std::str.upper.full(lend $s, lang: std::casing.tr);
```

- **Your own kit's names stay bare** (`helper[]`; `my.helper[]` where needed), and so do the base's: `+`, `plus`,
  `str.utf8`, `ptr`, `to.`.
- **std is another kit**, so `list` and `map` are written `std::list[i64]`, `std::list.append[…]` (owner: "(a)"), and
  printing, `round.X`, `format`, `duration` and the text operations the same.
- **An enum value takes the path even where its type is known**: `lang: std::casing.tr` (owner: "no" to an
  exception).
- **A kit's operators, word and glyph, take an optional path** ("It's always optional", "glyphs are optional
  too"): `'a' merge 'b'` or `'a' greet::merge 'b'`, `'t' <+> 'u'` or `'t' greet::<+> 'u'`. Writing the path is how a
  clash between two kits' operators is fixed.
- **The `bare.kit_prefix.X` and `bare.enum_prefix.X` marks are gone** (2026-10-06 to 2026-10-22): a mark left in a
  kit is refused, naming this rule.

**`use`** (owner, 2026-10-22; Kotlin's shape: "I like how diabolical Kotlin's imports look"), one per line at the top
of a file:

```
use std::ls as ls;                  # list[i64], list.append[…], list.length[…] in this file
use std::print.shortest as say;           # say[…]
use greet::formal as formal;
use std::print.stored as std::print;    # a rename onto a path is allowed ("it's their choice")
```

- **`as` is always written**; the name after it stands for the path in that file. A path names a function, a module
  or a type.
- **Several in one line go in `[ ]`** (owner, 2026-10-24: "If use doesn't support doing a lot of shit in one line, it
  should", then "Yes"): `use [std::print.useful as printu, std::list as list, std::str.join as join];`, each with its
  `as`; one alone may go bare or in brackets. `examples/use_list.cy`.
- **Every `use` gets a note in the build report** (kayo's), saying what the name stands for.
- Design claude's readings, not asked: `use` lines come before anything else; there is no wildcard; the path is
  another kit's (`use my.…` is refused); nothing in the file may be declared under a `use`'s name.
- `examples/use.cy`, `examples/uses_paths`; reject tests `std_name_needs_path`, `std_list_needs_path`,
  `std_duration_needs_path`, `kit_name_needs_path`, `bare_never_without_path`, `bare_marks_gone`, `use_*`.

## A keyword may name a function — reached by path, never bare (decided 2026-09-19)

`::` already says "what follows is a name in that Limb," so `std::to[…]`
is unambiguous even though `to` is a keyword; the reservation rule only
ever protected *bare* positions. So `fn to […]` is legal, and such a
function is reached through a path everywhere: `std::to[…]` from outside,
`my.to[…]` inside its own kit (`kit.to[…]` until 2026-10-06; owner floated `std:to` / `std:::to` for
the inside; `:` already attaches types and `limb.` already means "my own
strike," so `limb.` it is). Bare `to[…]` is the keyword. Structs, consts and
modules keep the strict rule — their names appear bare in type positions.

## `pub` and `const` (decided 2026-09-19)

```
pub fn add [let $a: int.sign.32, let $b: int.sign.32] -> int.sign.32:
    …
end of fn add
pub rec Locker:
    …;
end of rec Locker
pub const $max_players := 8;
const $pi: float.bin.64 := 3.141_592_653_589_793;      # private to the Kit
```

- **`pub`** is a bare word before `fn`, `struct`, or `const` at the top
  level: visible to other kits. Without it, private to this kit. `MAIN`
  is never `pub`.
- **A kit's top-level value is a `const`**, not a variable (a variable
  is refused at the top level). It is a value known at compile time — a
  literal, arithmetic on literals, another const, a pure call; anything
  with an effect or a runtime input is a compile error. It has no storage:
  the prover bakes it in at every use. It is never assignable, never `mut` /
  `!mut`, never shadowed, and no ownership verb applies to it — nobody
  owns it, every use is a fresh value.
- **Named like a variable, quoted**: `'pi'`, used in expressions as
  `'pi' times 2`, reached from another kit as `mathstuff::'pi'` (owner,
  2026-09-19: "single quoted"). One rule — quotes mean a user-named value.
- **The OS door is open to every kit** (2026-09-19): `extern`, `ptr[T]`
  and inline asm are base-language features (`ffi.md`); std is the
  library everyone uses for them, not a privileged one.
- **There is no public variable** (owner, 2026-09-19: "No public
  variable"). A kit exposes constants, functions, and structs; state
  another kit may read *and assign* does not exist. The uses a public
  variable serves have owned shapes: a configuration knob is a parameter or
  a struct field; a shared counter or cache is a value someone owns and
  lends mutably; a singleton is what `MAIN` owns and hands down. A global
  has no owner, so ownership, whole-program analysis, and automatic
  parallelism would all have a hole in it. Closed, not parked.
- **A `const` may also sit inside any body** (owner, 2026-10-10: "I want
  it.", then "`a`": any body, `MAIN` and functions alike). It follows the
  top-level rules — its value is known at compile time, a variable or a
  `std::` effect in it is refused, it is never assigned to or shadowed,
  nobody owns it — and it is seen only in its block. It reopens "no `const`
  keyword for locals" (2026-09-14): the prover still finds constants by itself,
  and `let` stays the usual declaration; the word is for a reader (and a
  check) that says "this is fixed when the program is built". `let immut`
  is the other thing: a value worked out while running that is never
  changed. Proposed: (a) any body, recommended by design claude; (b) only
  `MAIN`. `examples/local_const.cy`.

## Modules (decided 2026-09-19)

A kit has modules: named boxes inside it, each with its own namespace and
its own private/public line. Owner: "we should have modules"; then (c) of
file-only / block-only / both.

### Where a module comes from

- **Every `.cy` file in the kit is a module named after the file**:
  `shapes.cy` → `shapes`; a directory nests: `shapes/circle.cy` →
  `shapes.circle`. Nothing declares it — Synthia knows the files, and `ls`
  shows the tree.
- **`module name { … }` inside a file** nests one more module that has no
  file of its own — for a private helper namespace or tests beside the code.
  The keyword is `module`, not `mod`: `mod` is the remainder operator.
  There is no `module name;` include line (Rust's `mod x;` exists only
  because Cargo does not scan).
- A module cannot be both a file and a block.
- **The root is `kit.cy`** (kept over `root.cy`; the echo with the
  `.limb` archive is fine — that is the whole kit packed, this is the file
  inside it). Its items are the kit's top level, `geometry::…` from outside.
  `MAIN` lives there and nowhere else. A small kit is one `kit.cy`.

### Names — "whatever has most compat"

A module name, file or block, is a lowercase ASCII identifier: `a`–`z`,
`0`–`9`, `_`, starting with a letter. Not a keyword; not a Windows device
name (`con`, `nul`, `aux`, `prn`, `com1`–`9`, `lpt1`–`9`). A file in the
kit whose name does not fit is a compile error naming it, never silently
skipped. Why: macOS and Windows file systems are case-insensitive
(`Shapes.cy` and `shapes.cy` collide on some machines and not others),
and macOS normalises Unicode names differently from Linux.

### Reaching in — `::` crosses, `.` narrows, `my.` is the root

```
shapes.area(…)                 # from the root: a module's item
my.helper(…)                   # from inside a module: the root's item
my.shapes.circle.area(…)       # from inside any module: an absolute path
circle.area(…)                 # from inside `shapes`: its own child
geometry::shapes.circle.area(…)   # from another Kit
$r: shapes.circle.Circle      # a struct through modules
shapes.circle.$pi             # a const through modules
```

- A bare name is **this module's**. A dotted path is **relative to this
  module**. `my.` starts at the root (`limb.` until 2026-10-06, when the
  owner chose `my.`; owner chose (c) over lexical
  fallback and over the kit's own name — a root item added later cannot
  break a distant module, and a kit's source does not depend on its
  package name). Rust's `crate::`.
- From outside, the kit's prefix replaces `my`: `geometry::shapes.area`
  — the same path with the boundary marked.

### Visibility — three audiences

```
function helper []:
    …
end of function helper                 # this module and its descendants
public.kit function shared []:
    …
end of function shared        # the whole Kit, not outside
public function api []:
    …
end of function api                # other Kits too
public.kit module tests:
    …
end of module tests        # modules carry it as well
```

- Nothing: private to the module it is declared in, visible to that module
  and the modules nested inside it (so a block reaches its file's private
  helpers, and `my.helper` works from anywhere when `helper` is at the
  root).
- **`pub.kit`**: the rest of this kit. Owner chose the chain over Rust's
  `pub(crate)` — `( )` is arithmetic grouping only — and over having no
  middle level. `pub.kit` narrows `pub` the way `lend.mut` narrows `lend`.
- **`pub`**: other kits.
- Every module on a path is checked the same way: a `pub` item inside a
  private module is unreachable from outside that module's parent.
- A file module is public (provisional: a file has no declaration line to
  carry `pub.kit`; each item inside still carries its own visibility).

## One name, one thing (decided 2026-09-21)

A module and a type (struct, enum, interface) cannot share a name in a
kit: `module shapes { }` beside `struct shapes { }` — or a file
`shapes.cy` beside it — is a compile error, "rename one". Proposed
alternative: allow it and tell them apart by what follows the dot
(`shapes.'pi'` the module, `shapes.claimed` the type). Owner chose the
refusal: nothing to disambiguate, for the reader or for the prover.

## A shorter name for users — `pub fn area = shapes.circle.area;` (owner, 2026-09-23)

```
# geometry/shapes/circle.cy
pub.kit fn area [let $r: float.bin.64] -> float.bin.64:
    …
end of fn area
pub.kit rec Circle:
    var r: float.bin.64;
end of rec Circle

# geometry/kit.cy
pub fn area = shapes.circle.area;          # users write geometry::area[…]
pub rec Circle = shapes.circle.Circle;  # users write geometry::Circle[…]
```

Read it as: other kits get a function called `area`, and it is the one
written in `shapes/circle.cy`. The kit's author decides what comes
after `geometry::`, whatever the folders look like inside (owner:
"Doesn't the Limb maker say what goes after `::`?"). The line reads as
a declaration: the kind, the name users get, and `=` the path to where
it is written. That path is the ordinary one inside a kit, relative to
the module the line is in (2026-09-19): `shapes.circle` is how the file
`shapes/circle.cy` is named everywhere, and source never writes a
file name, since a module may also be a `module` block with no file.
The line adds a name only: no code is copied, and it is the same
`area`. If the original is `pub.kit`, only the short name exists
outside, so the folder layout stays private. If it is `pub`, both
paths work.

Proposed alternatives: none, where the path follows the folders and an
author wanting `geometry::area` puts it in `kit.cy` or writes a
forwarding function (design claude's first recommendation, for one name
per thing); `pub shapes.circle.area;`, refused by the owner ("I don't
think that makes sense, the syntax") because the new name is only
implied; `pub fn area from shapes.circle;`, English-like but unable to
give a different name; a file path, `"shapes/circle.cy".area`, after
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
# std/str.cy
pub fn split = split.split;      # users write std::str.split[…]
```

So a big kit keeps its grouping (`std::str.…`) while the code behind
it lives in whatever files suit it. Proposed alternative: only in
`kit.cy`, so a kit's whole outside face is in one file; but then
every short name would sit one level deep, and keeping
`std::str.split` would mean the real function has to live in `str.cy`.

Not yet ruled:
which kinds it covers beyond `fn` and `struct` (`enum`, `interface`,
`const`, `module`).

## A kit's name (owner, 2026-09-23: a)

The name lives in **the manifest** (`Synthia.ttsr` / `CyJet.toml`), beside the version and the
dependencies, and nowhere in source. Synthia reads it and hands it to
kayo with the rest of what it is building. Renaming the directory changes
nothing, and a kit's own code never names itself: it says `my.` (`limb.` until 2026-10-06).
Proposed alternative: `kit mathstuff;` at the top of `kit.cy`, which
would give the name two places to disagree.

**A dependency may have a MAIN** (owner, 2026-09-29: a): it runs only when
that kit itself is run, and is ignored when another kit uses it, so a
library can ship its own demo or tool.

## Case (owner, 2026-10-06; the marks removed 2026-10-27)

**Since 2026-10-27 every name is exact, and the case marks are gone** (owner: "No case shit anymore", then "correct").
`case.any`, `case.exact` and their older spellings are refused. Two names in a kit that differ only in case build, each
such pair a note in the build report (owner: "(c)"), and an error for a name not found suggests one differing from it
only in case: "no function `area`; did you mean `Area`?" (owner: "(a)"). What follows is the rule as it stood before.

Names are exact (`case.exact`) unless a kit's writer marks them
`case.any`: on kinds in `kit.cy` (`case.any [fn, struct, enum,
interface, const, case, field, var];`), on one item (`pub case.any fn
area …`), on a block module, or as a file's first line (`case.any;`).
Writing `case.exact` is allowed; marks that disagree are an error. A
`case.any` name matches in any case, by Unicode's full case folding, in its
own kit and from other kits. Spelled `case.insensitive` /
`case.sensitive` until 2026-10-10 (owner: "(a)", to shorten them), and
refused now with the new spelling. Keywords stay exact. It costs nothing at run time. DECISIONS 2026-10-06
has the rulings and the oracle's readings; its examples, `case_marks.cy` and `uses_case`, went with the marks.

## Open

- A file module's own visibility (provisionally public).
