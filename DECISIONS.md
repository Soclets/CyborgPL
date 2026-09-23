# CyborgPL v2 — Design decisions

Running log. Each entry: date, the decision, who proposed it, and who chose it.
Nothing here is final until the owner says so; open questions live at the bottom.

## Decided

### 2026-09-23 — Precog follows a loop up to 100,000 trips (owner)

Kespar's provisional number, asked by compiler claude once it cost
something. CCA now follows loops bounded by an outside value trip by
trip up to 100,000 before widening. That made builds slower (the list
benchmark about 4.5 minutes, the sieve about 1.5, the sum 7 seconds, on
programs of 12 to 30 lines), and lowering the number would speed builds
at the cost of exactness on long loops. Design claude recommended
keeping it, since the owner had said slow compiles are fine and only
the final program's speed matters. Owner: "keep it at 100k". The
2,000,000-across-the-program budget stays provisional.

### 2026-09-23 — Wrapping on mixed sizes: say which (owner: b)

From compiler claude, implementing exact math: `('a' + 'b') +~ 1` with
an `int8` and an `int16` is exact 1100, but wrapping needs one size.
Proposed (a) the bigger; (b) an error until written, `to.int16['a' +
'b'] +~ 1`; (c) the left operand's, the oracle's placeholder. Design
claude recommended b, since wrapping exists for hashes and counters,
where the size is the algorithm. CCA already did b. Owner: b. The
oracle applies it to `'a' +~ 'b'` of two sizes too. Shifts of a mixed
exact sum were not asked, and stay a provisional oracle pick. Asked in
the same message, the type of `let 'x' := 'u' - 10` was already in the
record (`precog.md`, "`:=` widths"): the narrowest type holding every
value (-10 to 245, `int16`), adjusted for speed, so a lone scalar at
the machine's width and a list element or field narrow.

### 2026-09-23 — Plain math is exact and checked where it lands; wrapping wraps at its values' size (owner)

Check claude's question (generator seed 90's neighbour): in `let 'big':
int64 = (20000 + 20000) + 'small';` with `'small'` an `int16`, which
size does the bare-number group take? The oracle used the stored size
and ran it, and CCA used the name beside and refused it (40000 does not
fit `int16`). Proposed (a) the name beside decides (check claude's and
design claude's recommendation); (b) the stored size decides; (c)
neither, refuse until `to.int64[…]` says. The owner said "the math was
done inside `'big'`", so b. Design claude pointed out b would move
wrapping too (`'x' +~ 'x'` on a `uint32` gives 3,705,032,704 at 32
bits, 8,000,000,000 at 64). Further proposed: (c) split, plain math by
the stored size and wrapping by the values; (d) bare numbers exact like
`:=`, a mix of sizes at the bigger. The owner asked whether Precog
helps: it removes (a)'s checks where it proves fits, but cannot choose
the meaning. The owner then asked "What has the best perf & safety".
Answer: plain `+ - x xx` exact, checked where the value lands (a typed
`let`, assignment, argument, return), with the machine's 64 bits the
only limit in between, and wrapping at its values' size. It is never a
wrong number, never an error for an answer that fits, and has the
fewest checks, since 16- and 64-bit adds cost the same. Owner: "yes,
that one". This reverses the seed-23 fix below ("checked where it
happens", a reading, never ruled): `'v' > ('v' + 'v')` now compares
true values, and `'u' - 10` under `:=` is -5. It also settles mixed
sizes. `operators.md`, "Plain math is exact". `examples/overflow_sites.cpls`
rewritten. The reject tests overflow_in_comparison and
overflow_below_zero are gone, replaced by overflow_lands and
overflow_past_64_bits.

### 2026-09-23 — The shorter-name line may stand in any module (owner)

Second of the three leftovers. Proposed (a) only in `limb.cpls`, so
every short name a Limb offers is in one file; (b) any module, where it
adds the name to that module (`pub fn split = split.split;` in
`str.cpls` gives `std::str.split`). Design claude recommended b. The
owner asked why. Under a, every short name is one level deep, and
keeping a grouping like `std::str.…` would force the real code into
`str.cpls`, which ties the file layout to the names again. Owner: "in
any file". Left: which kinds beyond `fn` and `struct`.

### 2026-09-23 — Oracle bug: a comparison's typed side did not size a literal group on the other (verification session)

Generator seed 90: `let 'b': bool = 'n' > (0 -~ 1);` with `'n'` a
`uint64` was refused by the oracle ("`-~` wraps modulo a type's size,
so it needs to know which") and built by CCA, which was right. The fix
for seed 93 had stopped the stored `bool` at the comparison, but the
width did not then pass from one side of the comparison to the other,
as the rule says it does (88da73f: "any operand whose type the compiler
can work out"). Fixed. `examples/context_stops_at_a_comparison.cpls`
pins it.

### 2026-09-23 — The shorter name may differ: `pub fn new_circle = shapes.circle.new;` (owner: a)

First of the three leftovers from `pub fn area = shapes.circle.area;`.
Proposed (a) the name before `=` is free; (b) it must match the
original's, so `pub fn size = shapes.circle.area;` is an error and an
author with two `new`s renames them in their own files first. Design
claude recommended a: the line shows both names, and the author owns
both sides anyway. Owner: a. Still open: whether the line may stand
outside `limb.cpls`, and which kinds beyond `fn` and `struct`.

### 2026-09-23 — A shorter name for users: `pub fn area = shapes.circle.area;` (owner: b, then a, then a)

Re-export, left open when bare names were dropped: can the author of
`geometry` let users write `geometry::area` for the `area` written in
`shapes/circle.cpls`? Proposed (a) no: the path follows the folders,
and an author wanting a short name puts the item in `limb.cpls` or
writes a forwarding function that Precog removes; (b) yes, with a line
in `limb.cpls`. Design claude recommended a, for one name per thing and
because a shortcut can be added later but not taken away. The owner
asked "Doesn't the Limb maker say what goes after `::`?". Both options
let the author decide; a ties it to the folders and b frees it. The
owner asked how b would look. The first spelling, `pub
shapes.circle.area;`, the owner refused: "I don't think that makes
sense, the syntax". Proposed next: (a) `pub fn area =
shapes.circle.area;`, the kind, the name users get, and where it is
written; (b) `pub fn area from shapes.circle;`. Design claude
recommended b. Owner: a. The owner then said the path "doesn't seem
like it's referencing a file". It is the ordinary module path, the one
a call already uses (`shapes.circle.area[2.0]`), and source never
writes a file name since a module can be a block. Proposed (a) keep
the module path, (b) a file path, `"shapes/circle.cpls".area`. Owner:
a. The line adds a name only (no copy, the same item), and an original
marked `pub.limb` is reachable from outside only by the short name.
Not ruled: a different name from the original's (since ruled: a, above),
whether the line may stand outside `limb.cpls`, and which kinds beyond
`fn` and `struct`.
`limbs.md`, "A shorter name for users". The oracle refuses the line as
"not yet": it runs one Limb, where the short name has no outside to
serve.

### 2026-09-23 — Operators from another Limb: found by type, a clash is an error (owner: a)

Left open when bare names were dropped. An operator has no place for a
path, so `'u' dot 'v'` finds the `dot` written for its operand types,
with nothing written. The question was what happens when two Limbs
share the same operator for the same types, for example `<+>` on two
`int32`s. Proposed (a) allow any Limb to share an operator for any
types, and make a clash a compile error; (b) only the Limb that owns
one of the operand types may share an operator, as in Rust, so a clash
cannot happen. Design claude recommended b. The owner asked what a
custom operator on the language's own types is for: `~=` for decimals
equal within a margin, saturating `+|`, `rotl`, path joining `</>`,
`divup`. Under b a Limb could share those only as functions with a
path. Owner: "shouldn't two meanings of `~=` just err?", then a. With
no path to pick one, the fix for a clash is dropping a Limb or copying
the operator into your own code. `operators.md`, "From another Limb".

### 2026-09-23 — Three cores per session, one for the desktop (owner)

The owner asked how many cores the machine has, and said "Split it
evenly". Compiler claude proposed 3 each for the three sessions that do
heavy work, plus 1 spare, and relayed it. The owner confirmed it here
directly ("i did"). `oracle/.cargo/config.toml` sets `jobs = 3` and
`RUST_TEST_THREADS = 3`, and `CLAUDE.md` states it for every session.
The 10 are 4 performance and 6 efficiency cores, and macOS picks which
runs where, so this caps how many jobs a session runs at once. It does
not give each session an equal share of the fast cores.

### 2026-09-23 — `to.array[…]` / `to.list[…]`; a list never becomes an array by itself (owner)

The oracle let a list be given into an `array` variable and back, and
the record said nothing about it. The owner asked "don't we usually
have converters for these?" Yes: "nothing converts by itself"
(2026-09-19) already covered it, so the oracle was wrong. The only new
part was the spelling, and design claude proposed `to.array[give 'l']`
and `to.list[give 'a']` with the element type left unwritten, since it
does not change. `give` hands the elements over with no copy, and
`copy` keeps the original. Owner: "that looks right". The oracle now
refuses a list where an array is declared (a store, an argument, a
return) and the reverse, including an array's slice stored as an array.
A literal or a `:=` collection is neither yet, so it still goes either
way. Asked next: `to.array['l']` with neither `give` nor `copy`, an
error or a quiet copy? Owner: a, an error ("say `give 'l'` or `copy
'l'`"). A `lend` inside is refused too (owner: "aren't you supposed
to use another ownership word?"): a lend only lets you look, and a
conversion takes, so it says `copy`. `examples/list_array.cpls`,
four reject tests.

### 2026-09-23 — No bare names from another Limb; `pub bare` is dropped (owner)

Straight after `pub bare` was chosen, the owner asked why anything
needs to be bare when writer-made operators already are. Design claude
checked: an operator is always written bare (there is nowhere to put a
path in `'u' dot 'v'`) and is picked by its operand types, so a Limb
already has short syntax for two-operand work on its own types. Bare
would have added single-argument and many-argument calls, structs and
consts, at the cost of the only break in "the reader always sees where
a thing comes from" (2026-09-14), clashes between Limbs, and four open
follow-ups. Proposed (a) keep `pub bare`; (b) drop bare, so everything
from another Limb carries its path and only operators come bare.
Recommended b. Owner: "`mathstuff::pow` is fine", and when asked to
confirm, "drop bare". This reverses 2026-09-19's "the Limb maker
decides bare reachability" and the `pub bare` entry below. The oracle
refuses `bare` after `pub` with a message saying so.
`examples/bare.cpls` is gone, and one reject test replaces three. Left
open: how an operator from another Limb is found (by operand types, not
yet asked), and re-export (both since decided, above).

### 2026-09-23 — A whole-type range is allowed: `int32[min, max]`, `uint8[0, 255]` (owner: b)

Both mean the plain type. The oracle had refused `int32[min, max]` as
"`int32` written the long way". That was design claude's suggestion,
recorded as a ruling without the owner ever answering it (corrected in
eab52cc). It also accepted `uint8[0, 255]`, the same range in numbers.
Proposed (a) refuse both, since a range should say what the type does
not (recommended); (b) allow both. Owner: "they're harmless. So, no
problem." The refusal and its reject test are gone, and
`examples/one_ended.cpls` stores through both.

### 2026-09-23 — A range's bounds must fit its type: `uint8[0, 300]` is an error (owner)

The oracle accepted `uint8[0, 300]`: `input.md` said bounds are
literals of the base type's *family*, which rules out `int32[0.0, 1.0]`
but not 300 on a `uint8`. Proposed (a) an error; (b) allowed, the same
as `uint8[0, max]` since the type caps it anyway. Design claude
recommended a. Owner: "does `uint8` hold 300?", which was taken as a.
Each bound is now a value the base type can hold: `uint8[0, 300]`,
`uint8[-1, 5]` and `bin16[0.0, 70000.0]` are refused with "300 does not
fit uint8". Two reject tests.

### 2026-09-23 — A bare name is `pub bare fn pow […]` (owner: a) — dropped the same day, see above

The spelling of the Limb author's choice, open since 2026-09-19 (the
author decides whether users may write `pow[…]` without
`mathstuff::`). Design claude first proposed `pub.bare`, `pub bare` and
a list in `limb.cpls`, and recommended `pub.bare` to match `pub.limb`.
The owner asked whether bare is really a narrowing of `pub`. It is not:
`pub.limb` changes *who* can see a name, while bare changes *how* they
write it and leaves the audience as it was. A dot picks one variant
from several, and bare sits on top of visibility instead. `pub.bare`
was withdrawn. The two left were a separate word on the item, `pub
bare fn pow […]`, and a list at the Limb's front door, `bare pow,
sqrt;` in `limb.cpls`. Recommended the word on the item, so the reader
sees how `pow` is reached where `pow` is written. Owner: a. `bare`
without `pub` is refused, since a private name is already bare where
it can be seen. `pub.limb bare` is refused as undecided. `bare` is not
a keyword: it means this only right after `pub`. The word was once
floated for a shelved variadic parameter flag (2026-09-19), which would
need another name if it came back. `examples/bare.cpls`, three reject
tests.

### 2026-09-23 — Oracle bug: `min`/`max` read by side, not by word (compiler session)

`let 'v': int32[max, 5] = 5;` ran, and so did `int32[max, max] = 5`
and `int8[min, min] = 5`. The oracle took a bound word on the low side
to mean the type's bottom whatever it said, and one on the high side
to mean the top, so `[max, 5]` became `[min, 5]`. The owner's rule is
that `min` and `max` are the base type's own limits, so the word says
which one. Fixed: `int32[max, 5]` is refused as low-first, and
`int32[max, max]` holds only 2147483647. Two reject tests. CCA already
read it this way.

### 2026-09-23 — A Limb's name lives in `Synthia.cplm`, not in source (owner: a)

Open since 2026-09-14. Proposed (a) the manifest carries the name,
beside the version and the dependencies, as Cargo does; (b) `limb
mathstuff;` at the top of `limb.cpls`, so the compiler knows the name
without a build tool. Design claude recommended a: one place for the
identity, and nothing in source to disagree with it. Owner: a. The
owner asked who reads it: **Synthia** reads the manifest and hands the
name to CCA when it calls it, as it hands over the target. CCA never
reads configuration, and a Limb's source never depends on its own name
(`limb.` already means "my own Limb", 2026-09-19).

### 2026-09-23 — Oracle bug: the context type leaked into a comparison's operands (verification session)

Generator seed 93, shrunk by hand: `let 'b': bool = (5 -~ 'w') > 1;`
with `'w'` a `uint32` was refused by the oracle ("`-~` wraps modulo an
integer's size, and bool is not an integer") and built by CCA, which
was right. This is the family of seed 41, reached through a comparison
this time rather than a call. The rule was already there: the width
comes from either side of an operator, and the `bool` holds only the
answer. Fixed: the context stops at a comparison too. `operators.md`
says so beside the call rule, and
`examples/context_stops_at_a_comparison.cpls` pins it. The seed-23
check (an `int8` sum overflowing inside a comparison) still fires.

### 2026-09-23 — Commit subjects: `kind.folder.narrowing: sentence` (owner)

Every commit's first line says what kind of change it is, then where,
then optionally something narrower, with dots, as the language narrows
(`str.each.point`, `lend.mut`), and never more than three words deep:

```
decision.slicing: list.slice / array.slice, owning its elements
design.precog: the proof cache key, written down
feature.oracle.map: map.keys / map.values / map.pairs
fix.cca.precog: the interval for a widening loop
test.check: seed 23, arithmetic unchecked in a comparison
house.license: Apache-2.0 WITH LLVM-exception
```

The kinds: `decision`, `design`, `feature`, `fix`, `test`, `house`.
The second word is the **folder** (`cca`, `synthia`, `oracle`, `check`,
`examples`) for code and tests, and the **topic** for `decision` and
`design` (they all live in `design/` and `DECISIONS.md`, so the folder
would say nothing) and for `house` (`house.license`, `house.readme`,
`house.brand`, which covers the logo, names and trademark).

Owner asked for organised categories like `<category>:`. Proposed first
were plain prose, Conventional Commits (`feat(scope):`), and one word
per repo area (`oracle:`, `cca:`). The owner suggested narrowing with
`.`, then kind first (`feature.compiler.precog`, `fix.compiler.precog`).
Design claude proposed `decision` as its own kind rather than
`feature.decision`. For upkeep the owner wanted a word other than
`repo`; the proposals were `house`, `meta`, `project`, `chore`, `brand`,
and the owner chose `house`. For the second word, folder names over
plain words (`compiler`, `build`) (owner), and `house.brand` over
`house.logo` (owner). The trailer still says which session wrote the
commit. Earlier commits keep their prose subjects.

### 2026-09-23 — List slicing: `list.slice` / `array.slice`, owning its elements (owner: a, a)

Surfaced by the `range`-as-a-value question: text had
`std::str.slice.grapheme` and lists had nothing, so taking part of a
list meant a loop and appends. Proposed (a) a slice is a new list
owning copies; (b) a borrowed view, as Rust's `&[T]` and Go's slices.
Owner asked whether Precog could help — it is exactly what decides it:
under (a) the copy is the meaning but is elided wherever nobody can
tell, giving (b)'s machine code for the common read-and-drop, while (b)
would add a third category beside owned and lent and refuse programs
that escape a view. Owner: a. Second question, whether `array` gets it
too: yes (owner) — slicing reads, so the reason an array has no
`append` does not apply. The option the owner picked read "both,
each giving a `list[T]` (or an `array[T]` where Precog proves the
length)", so **an array's slice is a list**. That half was left out of
this entry and added the same day, after compiler claude asked the
question again. 1-based, inclusive, positions say `pos`, a
check site rather than a failure since it is a language operation.
`examples/slicing.cpls`, two reject tests.

### 2026-09-23 — `range[…]` stays part of `for`; it is not a value (owner: a)

Open since 2026-09-19. Owner asked what a range value is even for, and
whether there are alternatives: elsewhere it serves slicing, passing
bounds, looping twice, and iterator chains — and here those are
`list.slice` / `std::str.slice` with `pos` arguments, two parameters or
a writer's own `struct Span`, writing the loop twice, and nothing,
since there are no iterator chains. A range value would also drag in
questions (a type parameter? lendable? what is its length?) for a
feature nothing has asked for. Owner: a. The exercise surfaced a real
gap instead: **list slicing does not exist**, where text has
`std::str.slice.grapheme`. Taken next at the owner's direction.

### 2026-09-23 — One-ended ranges: `int32[1, max]`, `int32[min, 100]` (owner: a, words)

Open since 2026-09-20. Today a one-ended bound had to be written with
the type's own limit as the other end (`int32[0, 2_147_483_647]`),
which is noise and silently wrong if the width changes. Proposed (a)
add them; (b) leave it, since `uint32` already says "not negative".
Owner: a. The spelling: `…`/`...` were proposed and withdrawn — the
owner pointed out `...` is the variadic and the spread, and the
one-character ellipsis cannot be typed — then `*` against words;
owner chose **words**, matching `x`, `mod`, `and`, `shl`. `min` and
`max` are the base type's own limits, so `int8[1, max]` and
`int32[1, max]` differ correctly from one line. `int32[min, max]` is
refused as `int32` written the long way. That refusal was design
claude's, suggested beside the question and never put to the owner.
It was recorded here as if ruled, and corrected the same day (see
"Is a whole-type range refused?" under Open). In the oracle the same day;
`min`/`max` survive into messages, so a refusal reads "0 does not fit
int8[1, max]".

### 2026-09-23 — Oracle bug: the context type leaked into a call's arguments (verification session)

Generator seed 41, shrunk: `let 'v': uint8 = g[0 > (-1) - 'x'];` with
`'x'` an `int64` was refused by the oracle ("-1 does not fit uint8")
and built by CCA, which was right. Introduced that morning with
cc04fb9: the type a value is stored into was being carried into every
nested expression, including a call's argument list, where the
*result's* type says nothing about the arithmetic in the arguments.
Fixed: the context stops at a call, a conversion, a construction, a
lambda or a map/enum/channel literal. Recorded in `operators.md` beside
the width rule; `examples/context_stops_at_a_call.cpls` pins both
halves — the argument that must not inherit, and the literal that must.

### 2026-09-23 — Widths confirmed, minus `bin128` (owner: b)

Marked "proposed by Claude; owner to confirm" since 2026-09-14 and
built on ever since. Confirmed: `bin` 16, 32, 64; `deci` 32, 64, 128
(the standard defines only those three); `int`/`uint` 8, 16, 32, 64,
128. Proposed (a) confirm all including `bin128`; (b) drop `bin128` —
the one with essentially no hardware anywhere, entirely software and
slow, and whose use case `std::uenr` answers by being exact rather than
wider. Owner: b, "but `std::uenr` better be fucking optimised" —
recorded in `types.md` as a requirement, since `uenr` is now the only
answer above 64 bits. `int128`/`uint128` stay: no hardware either, but
cheap from two 64s, and `uenr` is built on them.

### 2026-09-23 — No implicit zero: a declaration always has a value (owner: a)

Open since 2026-09-14 as Claude's recommendation, never put to the
owner. `let 'n': int32;` is refused; write the `0`. Proposed
alternative: fill in a zero by type (`0`, `0.0`, `false`, `""`, an
empty list), as Go, C# and Java do for fields. Owner: "just write
`0`, bruh". It also keeps `:=` honest — `let 'n';` would say nothing
at all — and leaves no place where a value appears from nowhere for
Precog to reason about. The oracle already enforced it.

### 2026-09-23 — Still no `std::`: finish the base first (owner)

Raised because thirteen examples print with `std::printu` and so cannot
reach machine code, leaving the differential harness comparing a
fraction of the corpus; the narrow unblocking version would have been
`print`, `printu` and `exit` written in CyborgPL on top of `extern fn
write`. The owner asked what was left in the base — two confirmations
never actually put to him (no implicit zero value; the exact width
list), two small additions (one-ended ranges; `range[…]` as a
standalone value), and two Limb questions (whether a Limb declares its
own name in source; the spelling of the author's "reachable bare"
flag) — and answered: **finish the base first.** The 2026-09-22 rule
stands unchanged until those are closed.

### 2026-09-23 — The manifest is `Synthia.cplm`, TOML inside (owner)

`.toml` had ridden in with the JetPack sketch on 2026-09-14 and was
never put to the owner. Proposed: `Synthia.toml`; `Synthia.cplm`
(*CyborgPL Manifest*, matching `.cpls` and `.limb`); the manifest
written in CyborgPL itself. The owner asked whether `.cpm` was taken —
its uses are all outside developer tooling (Corel, Cisco, Adobe
Captivate, Turbo Pascal), though the *name* CPM belongs to CPM.cmake —
then chose `.cplm`, which turns up nothing at all and keeps the
three-letter family. Contents stay TOML: the extension says whose the
file is, the format stays standard. Also recorded: **Synthia because it
synthesises everything** (owner), replacing Claude's "Jet = fast, Pack
= package" gloss, which had outlived the name it explained.

### 2026-09-23 — The build tool is **Synthia**, not JetPack (owner)

Renamed because JetPack is already a software trademark twice —
Google's Android Jetpack and Automattic's Jetpack for WordPress — and
both are developer-facing software, close enough that an application
would likely be refused and a rename could be forced later, when the
name is in every doc, URL and config file. Owner asked whether a narrow
registration ("package manager for CyborgPL, nothing else") would help:
no, because examiners compare goods rather than audiences, and the
publication step is what puts a name in front of the other owner's
counsel. Claude then offered workshop words (Bolt, Socket, Torque,
Rivet, Ratchet, Chassis) and withdrew them — all crowded, and `Socket`
is a package-security company. Owner proposed **Synthia**; checked
free on crates.io and as a GitHub org. Its neighbours are SYNTHIA®
(Merck KGaA, chemistry software — registered) and Synthesia (AI video);
neither is in this lane. So: `Synthia.cplm` (see the entry above),
`synthia build`,
`synthia final`, `synthia run`, `synthia test`. Every design document
renamed; the entries above this one keep the old name as the record of
what was decided then.

Owner's position on marks, from the same conversation: **™ now, ® never
unless something changes** — ™ is free and asserts the claim,
registration is for enforcement a hobby language will not need.

### 2026-09-23 — Licence: Apache-2.0 WITH LLVM-exception throughout, plus trademark (owner: b)

The 2026-09-14 intent — free to use with no credit, credit everywhere
for a modified compiler — had proposed a non-standard "Apache plus a
CyborgPL attribution term" for the compiler. Proposed now: (a) that
term, which says literally what the intent asked but is not
OSI-approvable, is rejected on sight by many corporate legal
departments, and repeats the BSD advertising clause; (b) plain
Apache-2.0 WITH LLVM-exception everywhere, with trademark on the names
doing the work — Apache already requires credit in the source, and a
fork may not call itself CyborgPL or Precog. Owner: b — "not what I
originally wanted, but it's reconsidered". The cost accepted: a fork
need not credit CyborgPL in its docs or version output. No `LICENSE`
file yet; the text is still counsel's, and nothing is urgent while the
repo is private.

### 2026-09-23 — Oracle bug: arithmetic was checked only where it was stored (verification session) — reversed the same day: plain math is exact, checked where it lands (owner)

Generator seed 23, shrunk: `if 'v' > ('v' + 'v')` on an `int8` of 112
ran in the oracle (it computed 224 and compared it) and was refused by
CCA, correctly — `'v' + 'v'` is an `int8` addition wherever it stands,
and `precog.md` lists integer overflow as a check site. The oracle had
been checking width only on a store (a `let`, an assignment, an
argument), so the same sum inside a comparison went unchecked. Fixed:
`+ - x xx` on integers check their result against the operand's type
where the operation happens. It also catches `'u' - 10` on a `uint8`
going below zero, which was never caught before.
`examples/overflow_sites.cpls`, two reject tests.

### 2026-09-23 — A detached task's failure is handled inside the task (owner: a)

Compiler claude's gap: a normal task's `otherwise` runs when the block
that started it ends, and `concur.detached` has no such block. Proposed
(a) the handler runs inside the task the moment it fails — it can
print, log or retry, but cannot return or fail outward; (b) a detached
task may not fail at all; (c) its failure ends the program with the
reason named. Owner: a. The autosave that could not write is the reason
detached tasks exist, so it should be able to say so while the program
carries on.

### 2026-09-23 — A certain failure is an error only if the site is always reached (owner: c)

Found by the generator (seed 59): a program with no input at all, where
CCA refused a line inside an `else` the oracle shows is never taken —
Precog's sets had widened across nested calls. `precog.md` said "no
value legal → a compile error"; the tiers said the working build "never
refuses to compile". Proposed (a) refuse, reached or not; (b) refuse
only when the site is reached on every run, otherwise treat it as any
unproven site; (c) b plus a `note:` in the working build. Owner asked
why not (a), and whether the options really differ — they reduce to two:
refuse to build, or build and say something. The deciding argument:
under (a) the analyser's imprecision becomes the programmer's error,
and it bites exactly on programs big enough for Precog to widen. Owner:
c. Both builds stay safe — the final build refuses either way.

### 2026-09-23 — A shift's count must be 0 to width − 1 (owner: a)

Found by the generator. `let 'a': int8 = 'v' shr 8;` and `'v' shr (-1)`
were errors in the oracle and proven-safe zeroes in CCA. Proposed (a)
any count outside 0..width−1 is a check site for both directions; (b)
`shr` by the width or more is defined as 0, only a negative count
checked; (c) (a) for `shl`, (b) for `shr`. Owner: "why don't they just
use minus?" — which is the case for (a): `shr (-1)` means `shl 1`, so a
negative count is never deliberate, and shifting every bit out is never
how anyone asks for 0. Owner: a. Recorded with it, unwritten until now
though both engines agreed: `shr` on a signed value is arithmetic and
keeps the sign.

### 2026-09-23 — Oracle bug: two failing tasks in one block (compiler session)

Compiler claude, building `concur`: two `concur` statements in one
block, both failing, lost the enclosing scope when the second handler
ran ("'seen' is not declared"); one failing task was fine. Cause: a
task's body runs on its own scope stack, but the tasks waiting in the
*outer* stack were still in the same pending list, so the first task's
block drained the second and ran its handler against the wrong stack.
Fixed: a task's body starts with an empty pending list and gives back
whatever detached tasks it left. `examples/two_tasks_fail.cpls` (exit
11). CCA had it right; it found the same shape in its own MIR while
fixing it.

### 2026-09-23 — `examples/gradebook.cpls` stands; the sum template is Precog's job (Claude, from the record)

Compiler claude asked whether to narrow the `check` on `return 'total'
/ list.length[…]` or to treat sum/count invariants as Precog's work.
Answered from the record rather than the owner: the template list
decided 2026-09-22 already includes "a sum is bounded by trips times
the largest element", and that is exactly what makes an average fit its
element type — a sum bounded by n × max, divided by n, is at most max.
So the example is right and CCA has work to do; it stays in
`cca/compiler/tests/examples_unproven.txt` until the template lands.
Noted in `design/precog.md` beside the template. (The example had been
passing only because of a Precog bug CCA has since fixed, 61a8554: it
never forgot a list after `record[lend.mut …]`.)

### 2026-09-23 — A shift's width comes from any operand whose type is known (owner: a)

The generator's first-run finding, and two-thirds of its early
disagreements. `let 'a': uint32 = 2 shl 3;` and
`let 'b': uint32 = ('v' mod 16) shl 2;` ran in the oracle and were
refused by CCA ("`shl` only knows a declared width"). Proposed (a) any
operand whose type the compiler can work out is enough, a literal
taking its width from its context as literals do everywhere else; (b)
the left operand must be a name or a `to.T[…]`, so the width is visible
in the operation; (c) (b) for bare literals only. Owner asked what
`shl` is, had the two lines worked out (16 and 12), and asked why (a)
would not be possible — it is: the obstacle was never technical.
Owner: a. `operators.md`'s "the compiler knows every width; the oracle
needs the left operand declared" was always about the oracle's limit;
CCA had read it as a rule and is the one to change. The oracle also
stopped being the limit the same day: it now works the width out from
a name, a conversion, either side of an operator, or the type the value
is stored into, for `shl`/`shr`, `not.bit` and the wrapping forms.
`examples/widths.cpls`.

### 2026-09-23 — Generated programs read input as an environment variable's length (owner: a)

Check claude's second generator question, the same shape as the
printer. Without input, every generated program is fully known and
Precog runs it to the end at compile time, so nothing tests the machine
code or layers 2 and 3. Proposed (a) `extern fn getenv` + `strlen`,
with the harness setting the variable's length and the program building
its values from it, switching to `std::input` when std has one; (b)
wait for std. Owner: a. Probing this by hand had already found three
CCA bugs that day, including a wrong proof (`'n' x
10000000000000000000` into a `uint64` reported proven; the binary
overflows at n = 2 where the oracle stops) — the silent-miscompile
class, and the argument for the whole session.

### 2026-09-23 — Generated programs carry a base-language printer (owner: a)

Check claude's first generator question. With `std::` refused until std
exists, a generated program has no way to print, so the harness could
only compare exit codes. Proposed (a) each generated program carries a
small printer written in the base language — ASCII digits into a
`list[uint8]`, then C's `write` under `nocheck`, as `examples/ffi.cpls`
does; check claude wrote one and the oracle and CCA printed `0`,
`12345` and `-907` identically; (b) wait for std's print. Owner: a. The
printer is ordinary CyborgPL, so a bug in it is itself a finding; when
std has a print the generator switches to `std::printu`.

### 2026-09-23 — A `for` range's step is its own signed number, not one of the counter's values (owner: a)

The verification session's first finding, on its first day: for
`for 'i': uint64 in range[3, 1, -1]` the oracle failed at run time
("-1 does not fit uint64") while CCA proved all four sites and ran the
loop. `design/control-flow.md` said "every value must fit" and that a
negative step counts down, but never what type the step has — so
neither was wrong, and check claude asked rather than filing a bug.
Proposed (a) the step is its own signed value and only the counter's
values must fit — CCA's reading, check claude's and Claude's
recommendation; (b) the step must fit the counter's type, which makes
a backwards walk by position impossible since a position is a
`uint64`. Owner asked how a `uint` can use a minus at all — it never
holds one; the step is an instruction, and every value the counter
takes stays ≥ 0. Owner: a. Noted with it: the compiler must compare
before stepping, or `range[3, 0, -1]` underflows at the bottom. The
oracle was the one wrong and is fixed the same day (it was fitting
every bound, step included, to the counter's type);
`examples/countdown.cpls` covers all three shapes.

### 2026-09-23 — The verification session opens: "check claude" (owner)

Owner: "let's start the verification session". Its brief is
`design/verification-session.md`, its clone
`/Users/ts/CyborgPL v2 verification`, its code `check/`, its trailer
`check claude`. The rule that gives it its point, from 2026-09-21: it
is a different author from Precog's, so it never reads `cca/` or
`oracle/src` — it attacks what they claim, never their reasoning.
Build order: the generator, the differential harness, the Precog
oracle's three attacks, then translation validation on Alive2.

### 2026-09-23 — `closed` is counted at run time, and Precog deletes the counting where it proves it (owner: c)

The last thing blocking `channel.take … otherwise` and `whichever`.
Proposed by the compiler session: (a) a channel counts the tasks that
can reach it and fails `closed` when every remaining holder is itself
waiting to take from it — exact, two counters per channel; (b) Precog
emits the close where structured concurrency proves the last sender is
done — free where it works, not every case; (c) both, (a) built first.
Compiler claude recommended c, Claude agreed, owner: c.

### 2026-09-23 — A channel's slots are inline; Precog stacks, shrinks or removes it (owner)

The compiler session's remaining layout question, deferred until
`concur` generated code. Proposed (a) slots inline in one block, since
the capacity is always known when the channel is made; (b) a separate
heap buffer behind a pointer. Owner asked whether Precog helps, then
for the performance difference; measured on the M5 in C with
optimisation barriers: inline versus pointer access is 2.44 ns against
2.47 ns (nothing — the reason for inline is one allocation instead of
two), heap creation is 13–19 ns against 0.9 ns on the stack, and
removing the channel entirely is 2.7× on the operation and more in
reality, since a real send and take also pay atomics and a scheduler
round trip. Owner: a plus Precog — inline slots, and Precog puts the
channel on the stack when structured concurrency proves it dies with
its block, allocates only the slots it proves are needed, or removes
a one-value hand-off altogether.

### 2026-09-23 — This session signs as `design claude` (owner)

The trailer named the model (`Claude Opus 5`), which goes stale the
moment the model changes — as it was about to, with Opus 5.5 out.
Owner: "yours should be `design claude`", matching the compiler
session's `compiler claude`. A house name says which *session* wrote a
commit, which is the thing the log is for; the model is not in it.
Earlier commits keep the old trailer as a record of what wrote them.

### 2026-09-22 — Map layout: two shapes, ordered and unordered (owner: b)

The fourth layout question. Open addressing was already decided; the
open part was the order list. Proposed (a) one four-field handle with
a null order pointer for unordered maps; (b) two shapes — three fields
unordered, four ordered — since Precog chose the variant and knows
which each map is. Owner: b.

### 2026-09-22 — Layout: `list` 24 bytes, `array` and `str` 16; Precog drops a proven list's capacity (owner)

The third layout question. Proposed (a) different shapes — each type as
big as it needs; (b) one 24-byte handle for all, simpler in the
compiler, 8 unused bytes on every text. Owner asked whether Precog
helps here: less than in the previous two, because the type already
says who grows — but it chooses the type (`:=` with no append becomes
an `array`), and a `list` whose final size it proves needs no capacity
field either, since the capacity is then a constant. Owner: a, plus
that.

### 2026-09-22 — A constant text: Precog knows which memory, a sentinel only where it cannot (owner: c)

The second layout question. A text handle can point at the binary's
read-only data or at the heap, and freeing the first would crash.
Proposed (a) a sentinel in the handle — the high bit of the length —
tested at every drop; (b) copy every constant to the heap on first
use; (c) Precog decides: a free is emitted only where the value can be
heap, with (a) as the fallback for a name that may hold either. Owner:
c.

### 2026-09-22 — `str` is a 16-byte handle; Precog adds a capacity where a builder is proven (owner: c)

The compiler session's first layout question (it blocks about three
quarters of `examples/` from reaching machine code). Proposed (a)
pointer + length, 16 bytes — nothing grows a text in place, so a
capacity is dead weight everywhere; (b) pointer + length + capacity,
24 bytes. Owner asked the difference and whether Precog could help,
then chose **c**, which that question produced: the type is 16 bytes,
and where Precog proves a text is built by repeated `++` in a loop
that text grows in place with a hidden capacity — one allocation
instead of N, paid only by the values that earn it. Replaces Claude's
2026-09-14 note in `design/types.md` (24 bytes), which was never a
decision.

### 2026-09-22 — A `std::duration` is never negative (owner: a)

The compiler session found `oracle/tests/reject/duration_negative.cpls`
enforcing a rule that appears in no document — Claude had put it in the
oracle when implementing durations and never asked. Proposed (a) a
duration is a length and never negative, subtracting past zero refused;
(b) signed, `5 seconds - 6 seconds` is `-1 second`; (c) no subtraction
between durations. Owner: a. "How far past the deadline" is the
difference of two instants, a type not designed.

### 2026-09-22 — A `nocheck` that certainly fails gets a note, not a refusal (owner: c)

Asked by the compiler session, which had Precog layers 1 and 2 running
and found that `nocheck let 'q' := 1 / 'z';` with `'z'` proven `{0}`
leaves the compiler knowingly emitting a program that is undefined
every run. Proposed (a) leave it — `nocheck` is "on my word" and the
site is already listed; (b) refuse when the failure is certain; (c) a
`note:` and build anyway. Compiler claude leaned c over b; Claude
agreed; owner: c. Precog never goes looking — it reports only what its
ordinary work already showed, and a `nocheck` that fails on some
inputs still ships silently.

### 2026-09-22 — Precog layers 1 and 2 in CCA (compiler session)

296 of the 337 check sites in `examples/` proven away; 19 reject
programs now refused with a counterexample; `examples/pi.cpls` runs
its 60-digit spigot at compile time and proves all 43 sites. Two
findings worth the record: `gradebook.cpls`'s comment ("the if above
proves the divisor is not 0") required narrowing to reach back through
`list.length[…]` and a struct field — the example had been asserting a
capability that is now real; and trip-by-trip loop following is not
only where compile time goes (Kespar's note) but where the proofs come
from — joining at the loop head cannot see a counter and a sum move
together, so `scores.cpls` went from 4 of 9 sites proven to 9 of 9,
and from 1675 steps to 170.

### 2026-09-22 — The same fault twice, and a hole in the oracle (compiler session)

After the ruling below, the compiler session found `check_tiers.cpls`
line 22 doing the same thing (`'xs'[pos 1] = 'x' / 2` — a bin into an
integer list) and diagnosed why the oracle had not caught it: element
assignment enforced the element type only when the list was
*declared*, never when it was `:=`. Both fixed: the oracle now takes an
untyped list's element family from what is in it (reject test
`bin_into_untyped_int_list`), and the example spells the crossing —
`to.int32['x' / 2.0] otherwise 0` — which keeps the output byte for
byte and demonstrates the rule the example is now about. Two
independent readings of one spec, working as intended.

### 2026-09-22 — A name never changes numeric family; `examples/check_tiers.cpls` was wrong (owner)

The compiler session, building its checker, found `design/types.md`
("a *variable* never converts: `let 'b': bin64 = 'a';` with `'a'` an
`int32` is an error", with the reject test `int_into_bin_var`) against
`examples/check_tiers.cpls` line 7, which stored an integer-holding
`:=` name into a `bin16`. It had passed only because that branch never
runs; the oracle refuses the same line when reachable. Its proposed
reading — a `:=` name has no family until Precog picks one, so it may
land in a bin slot and the store is a fit check — would let Precog
change 70000 into 69984 (the nearest `bin16`) with nothing written.
Owner, on being shown the line: "shouldn't this error?" — yes. The
docs stand; Claude's example was wrong and is fixed (the value is a
`bin32` from the start, so the check site is a width narrowing, which
is what `core-features.md` meant). Crossing families stays
`to.bin16[…] otherwise …`, which fails rather than rounds.

### 2026-09-22 — SIMD in the oracle (Claude, per the decisions above)

`simd[T, N]` with the target's shapes enforced, lane-wise operators
and bitwise, comparisons giving `simd[bool, N]` (refused as a
condition), `fill`/`load`/`store`/`all`/`any`/`max`/`min`/`sum`,
`fill` taking its lane count from the declared type.
`examples/simd.cpls`, five reject tests. Two provisional picks listed
in the oracle's README: scalar broadcasting, and `sum` as a check site.

### 2026-09-22 — SIMD operators are the ordinary ones, element-wise (owner: a)

Proposed (a) every operator a plain `T` has, lane by lane, with
comparisons giving a `simd[bool, N]` (so a vector is never a single
`bool`), plus `[pos i]` for a lane, `simd.load` / `store` against an
array, and the folds `all` `any` `max` `min` `sum` `fill`; (b) named
operations only. Owner asked whether `or.bit` on vectors still means
what it means — yes, the same bit-for-bit OR, N lanes at once — and
chose a. `design/simd.md` written; open: more `T`, shuffles, masked
load/store, `simd` in structs and lists.

### 2026-09-22 — `simd[T, N]` only where the target has that register (owner: b)

Proposed (a) any plain numeric T and any power-of-two N, the compiler
splitting or padding to fit the machine — hardware stays in
`JetPack.toml`; (b) only what the target actually has. Owner: b —
"shouldn't they just use the normal way?": a program reaching for
`simd` is writing for a machine, and a type that pretends otherwise
lies; anyone wanting portability writes the plain loop and lets Precog
and LLVM vectorise it. So on ARM64 (128-bit NEON) `simd[uint8, 16]`,
`simd[int32, 4]`, `simd[bin32, 4]`, `simd[bin64, 2]` and the like;
`simd[uint8, 64]` is a compile error naming what the target has. A
`simd` program built for another target fails to build rather than
running slowly — the same honesty `inline.ASM` has.

### 2026-09-22 — SIMD types are spelled `simd[uint8, 16]` (owner)

Proposed (a) `uint8x16` (Swift's and Rust's look, a naming convention
the parser must know), (b) `simd[T, N]` — the language's own bracket
rule for type parameters, (c) `uint8[16]` (refused: it collides with
the range-type shape). Owner: `simd[uint8, 16]`.

### 2026-09-22 — Explicit SIMD types: language types like `uint8x16` (owner: b)

Raised by Swift's `SIMD16<UInt8>` example. Proposed (a) none — the
plain loop with its checks proven away is what LLVM's vectoriser
wants, measure first; (b) language vector types (`uint8x16`,
`bin32x4`) with element-wise operators; (c) a std Limb over
intrinsics. Claude recommended a; owner chose **b** (first said a by
misreading, corrected at once). To design: the spelling and which
widths, the operators, how a vector is loaded from and stored to an
`array`, one lane (`[pos i]`), what Precog tracks per lane, and what
happens on a target whose registers are narrower. Plain loops still
auto-vectorise; the types are for when that is not enough. Same day:
Swift 6.4 `-O` measured at 1.4–3.3× C on the seven benchmarks
(overflow and bounds checks it cannot prove away; refcounts; COW) and
0.9× on strings — the working-build cost Precog exists to remove.

### 2026-09-22 — No `std::` for now; Precog built in from the first commit (owner, in the compiler session; confirmed here)

Relayed by compiler claude on its first day, confirmed by the owner in
the design session. (The design session first told it the date was the
23rd; the compiler session had it right — the 22nd.) (1) **No `std::` for now** — the base language
first (the 2026-09-19 focus). Compiler claude had asked whether the
first milestone could treat `std::` names as compiler-known built-ins
lowered to runtime calls, or std be written in CyborgPL from day one;
owner: neither, yet. CCA refuses `std::…` as "not yet"; its own tests
print through `extern fn write` under `nocheck`, as `examples/ffi.cpls`
does. Consequence noted here: every example prints with `std::printu`,
so the differential test against the examples waits until some print
exists — the base-language-first order accepts that. The std Limb's
author and timing stay open. (2) **"Optimization shouldn't be bolted
on later, it should be while building"** — Precog and the proof-to-LLVM
path are designed in from the first commit: MIR carries domains and
check sites from day one, layers 1 and 2 run as the pipeline grows. The
brief's "a CCA whose Precog proves nothing" first milestone is
withdrawn.

### 2026-09-22 — The compiler session's brief; house name "compiler claude" (owner)

Owner asked what the brief would include, approved it, and added: the
design session's address (it is `SendMessage` to "CyborgPL language
design"), a normal `Co-Authored-By` trailer in the house name
**compiler claude**. `design/compiler-session.md`; `CLAUDE.md` lists
the three sessions. Where: the same repo, on `main` with no branch, in a second clone at
`/Users/ts/CyborgPL v2 compiler` — owner: "two local folders, because
same folder would share memory" (Claude's memory is per folder); code
in `cca/` and `jetpack/`; the compiler session never edits `design/`,
`oracle/` or `examples/`. (Claude flipped this twice on the way; this
is the settled form.)

### 2026-09-22 — Translation validation borrows Alive2; three sessions (owner)

Owner asked whether the compiler is buildable (yes; the undecided
items do not block it), which session does what (design + oracle
here; CCA + JetPack in a fresh session; verification in a third,
later), and about translation validation: why not written by us, and
Alive2's size — 37k lines C++, counted from a temporary clone. Claude's
"a year of expert work, less trusted" was an unsourced estimate and
said so when asked; the trust argument (years of LLVM developers'
use) stood. Owner: "Borrow it."

### 2026-09-22 — `std::printf`: C's print in full, checked (owner)

A third print where the text is processed. Claude proposed escapes
only (a) vs escapes plus `%` formatting (b); owner asked to see C's
list, then "full-on". Named `printf`; `%` checked against argument
types and count because the format is a literal; non-literal format
is a nocheck site; `%u` `%p` `%n` refused. Oracle: implemented;
`examples/printf.cpls`, five reject tests.

### 2026-09-22 — Restated (Claude, not a decision): `""` holds exactly what is stored

Claude wrote `"hello\n"` in an assembly example; owner asked whether
that works — it doesn't (seven bytes, the backslash kept), and
`"hello" ++ \n` is C's six. Claude then misread the owner's next
message ("adding a `""` is not literally print") as a request to
restate this rule and wrote it into `strings.md`; the owner meant a
new print (see Open). The restatement is true and stays; it was not
asked for.

### 2026-09-22 — `whichever`, not `pick` (owner)

Owner asked what `pick` is, why not `pick_first` (nothing else to
pick; no underscored keywords — a variant would be `pick.all`), and
for other words. Chose `whichever` over `pick`, `await`, `select`,
`wait`, `listen`, `first`. Renamed throughout; the oracle, example and
reject test follow.

### 2026-09-22 — Concurrency: `concur`, explicit captures, structured + `concur.detached`, `otherwise` on the task, bounded channels, `whichever`, duration literals (owner)

Eight questions. Model: tasks + channels with ownership (b) over
Precog-automatic (a) and async/await (c). The word: `concur` (owner
asked for it over `task`, weighed "agree", chose it; `concurrent` has
no standard abbreviation). Captures: explicit `with […]`, all verbs,
lend a check site (a) over give/copy only (b). Lifetime: structured
with `concur.detached` for program-lifetime work (c); owner asked
what needs detaching — a task whose starter must return first
(autosave from `setup`, a library's listener). A task's failure:
`otherwise` on the `concur`, fired at the block's end (a). Channels:
capacity by the bound rule, no bound = hand-off, unbounded refused
(b). Several channels: `whichever { 'v' from lend 'c' { … } … }` (a). Time:
std, with duration literals — owner: `[1 seconds]` not `[1, seconds]`
(the unit on the number, as a bound); units nanos…hours, more precise
"down to what a clock measures"; singular and plural accepted.
`design/concurrency.md`. Oracle: implemented the same day — a
cooperative demand-driven schedule; `examples/concur.cpls`,
`examples/whichever.cpls`, seven reject tests.

### 2026-09-22 — Diagnostics: root cause first, Rust-helpful, human and IDE locations (owner)

Owner: the report has a line per `for` (yes); and asked how error
messages should be. Claude's first draft was one-liners (the oracle's
style) — owner: "less useful than C's". Redone in rustc's shape: the
sentence, source lines with spans, a second span, `help:` with the fix
as code, counterexample as input values plus the reproducing command.
Owner added the rules: say what started the fail chain; be helpful
like Rust's; a human-readable location and an IDE-readable one.
`design/diagnostics.md`: root first, three kinds (error / check /
note, no warnings), plain words, dry errors and notes with
personality, JSON for editors.

### 2026-09-22 — GPU targets in v1: Metal and Vulkan (owner)

Proposed Metal only (testable here), Metal + CUDA, all three. Owner:
Metal and Vulkan — Apple plus the portable one; NVIDIA through Vulkan;
CUDA an addition later if wanted.

### 2026-09-22 — GPU data crossing: Precog keeps data on the GPU across loops (owner: b)

Proposed (a) copy per offloaded loop, (b) Precog decides crossings
from whole-program liveness — over once, back once, unless the CPU
touches it between, (c) a programmer hint. Owner: b. On unified memory
nothing crosses either way.

### 2026-09-22 — GPU offload: a body may hold plain values and failures (owner: b)

`design/gpu.md` opened. Proposed (a) plain values only, (b) plus
`fail` — an iteration records its index and reason by atomic minimum,
the runtime raises the lowest on the CPU after the launch, (c) plus
text. Owner asked the performance difference, then to test it: Metal on
the M5, same body with and without a fail path — within noise at 16M,
+0.0% at 64M, 256M and 1G iterations (8.6 GB of buffers), GPU 13–42×
one CPU core. Owner: b. Open: data crossing, targets, the report, the
threshold, a loop-nest model.

### 2026-09-22 — Map count bound `map[K, V][lo, hi]`; bounds inside compose (owner: yes)

The list rule applied to maps. Owner asked about the things inside:
K and V are ordinary types, so `map[str.utf8[1, 3 bytes], int32[0,
10]][0, 2]` checks each bound where it applies. Oracle: `set` now
checks key and value against K and V too. Example, two reject tests.

### 2026-09-22 — Map literal: `map["ann" = 5551, "bob" = 5552]` (owner: a)

Proposed (a) `key = value`, the struct construction's spelling; (b) a
list of pairs `map[["ann", 5551], …]`; (c) `->`. Owner: a. `map[]`
refused in favour of `map.empty`; a key twice in a literal is an
error. Three reject tests.

### 2026-09-22 — Walking a map: `map.keys/values/pairs`, insertion order; the variant is Precog's or chosen (`map.ordered` / `map.unordered`) (owner)

What a walk yields: key / pair / say which — owner: c. Order:
insertion (a), unspecified-but-deterministic (b), sorted (c); owner
asked the a/b difference and the runtime cost — measured (Rust
`HashMap` vs `IndexMap`, 5M entries): 13% slower lookups, +11 B/entry,
13× faster walks. Owner asked why not two types like Rust; Claude: an
unordered-but-walkable type has unspecified output (a hole in
differential testing); instead Precog drops the order list for a map
never walked, as with `list`/`array` — owner: yes, "but it should also
be choose-able" (list/array already are). Spelling: `map.ordered` /
`map.unordered` by the dot rule (a) over a second noun (b) — owner: a.
A `map.unordered` cannot be walked. Oracle: keys/values/pairs, the
variants, two reject tests.

### 2026-09-22 — Maps: a language type; missing key is a failure; keys by structure or a type's own `==`/`hash` with consistency proven (owner: a, a, c, yes)

Three questions and a consequence. (1) Where: the language beside
`list`/`array` (a), a std struct of lists (b), Rust's std-generic-with-
compiler-hashing (c) — owner: a. (2) A missing key: a failure with
`otherwise` (a), a check site guarded by `has` (b), a found/missing enum
(c) — owner: a. (3) Keys: plain and text only (a), plus structs/enums/
lists by structure (b), plus a type's own `op ==` and `fn hash` via a
`Key` interface (c) — Claude recommended b (custom hashing's classic
`==`/`hash` disagreement bug); owner asked b vs c, chose c. Consequence
put to the owner: make "equal keys hash the same" a check site Precog
proves — owner: yes. `design/maps.md`, `examples/phonebook.cpls`,
`examples/keys.cpls`, four reject tests. Open: walking a map and its
order, a literal, a length bound.

### 2026-09-22 — A length bound on text names its unit: `str.utf8[1, 40 graphemes]` (owner: c)

Proposed (a) bytes, (b) graphemes, (c) the unit is part of the bound —
`bytes`, `points`, or `graphemes` after the high bound, since the
language refuses a single `str.length`. Owner: c. No unit is a compile
error. Conversions into a bounded text fail `out_of_range`.
`examples/text_bound.cpls`, three reject tests.

### 2026-09-22 — Cold read of `precog.md`: reuse key corrected; proofs-as-assertions mode (owner: yes)

A fresh chat read the Precog document. Decided already: `rlimit` not
timeouts and pinned solvers (Determinism); generics proven after
monomorphisation (per-shape); a failed runtime check traps with the
site named. Real gaps: (1) the reuse key hashed raw MIR bytes — spans
shift on any comment — and the function alone, though per-call-site
analysis proves callees inside callers; corrected to canonical MIR +
Merkle hash of reachable functions + argument domains + target + Precog
and solver versions and configs. (2) `nsw`/`nuw`/`!range` make Precog
part of the trusted base — a wrong proof is silent UB; owner asked who
a proofs-as-assertions mode is for (authors, never writers) and said
yes: the Precog oracle's fourth attack. Also from the read: **Precog is the last
MIR pass** (owner: yes) — MIR is frozen after it. LLVM's narrow fact
vocabulary (owner asked "why not" hand everything over): every fact is
handed over wherever LLVM benefits — native flags always, `llvm.assume`
by measurement, a wrapper pass for facts worth more than LLVM offers.

### 2026-09-22 — Three IRs: AST, MIR, LLVM (owner: b)

Proposed (a) none of ours beyond the tree, AST to LLVM (Kespar); (b)
one mid-level IR — a CFG with explicit temporaries, checks, drops,
lends, failure edges, spans, carrying proven domains on its types; (c)
a high-level IR as well. Owner asked which is better regardless of
code volume: b — the typed AST already is the high-level IR, and
quality comes from MIR carrying proofs, not from another layer. Owner:
b. Reuse keys on MIR bytes.

### 2026-09-22 — Solver disagreement: run the counterexample, continue safe, report the solver (owner: c)

Proposed (a) refuse the build; (b) treat as unproven, continue; (c) run
the `sat` solver's counterexample at compile time to see which solver
lied, continue with the safe answer, name the culprit. Owner asked
whether running it at compile time is input — no: the reads are
replaced by the solver's values, nothing is read, deterministic. Owner:
c, and asked whether the writer is told to open an issue — yes, at
Cyborg's tracker with the query file attached. Owner then asked whether
it is necessarily a solver bug — no: both solvers got Precog's query,
so a wrong query makes a right solver look wrong; the report says
"solver or Precog", and which one is the Precog oracle's job.

### 2026-09-22 — A ranged extern return is a contract enforced as a failure (owner: c)

Proposed (a) nothing on the extern, a guard after each call; (b) a
trusted range, unverified; (c) a range checked at the door. Owner
asked whether extern's `nocheck` conflicts — it would with a *check*
(nocheck removes checks), so (c) is done as a *failure*: the call fails
`out_of_range` outside the range, `nocheck` does not touch failures,
the final build keeps it — the same rule as the input boundary
(`to.int32[0, 100][…]`; Kespar's "read contract is not a site"). Owner
asked safety and performance vs (a): identical; (c) is written once.
Owner: c. Oracle: ranged return allowed, `otherwise` required, plain
return still cannot fail. `examples/extern_range.cpls`, one reject test.

### 2026-09-22 — The Precog oracle: exhaustive execution + independent interval analysis + re-encoded negation (owner: b + c)

Proposed (a) exhaustive/random execution over small domains (Kespar's
generator); (b) a plus an independent simple interval analysis
(attacks layer 2); (c) a plus the negation of each solver-proven site
through a second encoding (attacks layer 3). Claude first recommended
c; owner asked c vs b + c; given Kespar's bugs were all layer 2, Claude
shifted to b + c. Owner: b + c. Different author than Precog, no shared
code.

### 2026-09-22 — Layer 3 solver: a Z3 + CVC5 portfolio (owner: b)

Proposed (a) Z3 only, pinned; (b) a portfolio of Z3 and CVC5, both to
full budget, results combined; (c) Z3 behind an interface. Owner: b.
Determinism kept: never first-to-answer; any `unsat` proves; both `sat`
→ Z3's counterexample by fixed order; both `unknown` → unproven. A
`sat`/`unsat` disagreement is a solver bug and open as to what the
build does beyond reporting it.

### 2026-09-22 — Precog tracks `std::uenr` as exact rationals plus a digit-count bound (owner: b)

Proposed (a) intervals over exact rationals, no overflow site, memory
not Precog's business; (b) a plus a digit-count bound carried through
arithmetic, with a warning (never an error or a check site) when it
grows past a threshold; (c) opaque. Owner asked the a/b difference
(silence vs an early "this will eat memory" note on `'x' = 'x' x 'x'`
in a loop) and chose b. (Claude wrote bare `uenr` in the question; the
name has been `std::uenr` since 2026-09-19 — owner reminded.) `deci`
is an integer-like domain at its fixed digit count.

### 2026-09-22 — Precog's text domain: length + exact values + character classes (owner: c)

Proposed (a) a length interval only (Kespar), (b) plus an exact set of
values while small so `'cmd' == "quit"` narrows, (c) plus character-class
facts so a `to.int32` after a digits check is proven not to fail. Owner
asked the b/c difference (conversions after validation come out proven
vs keep a failure path) and chose c. Each text operation carries a rule
for its result's class; no rule → "any text". The class list is open.

### 2026-09-22 — Loop invariants: Precog guesses from templates, else unproven (owner)

For a loop Precog cannot run to the end: (a) guess invariants from a
fixed list of shapes (counter, sum, monotone index, max/min, count) and
keep those the solver confirms inductively; (b) a plus a programmer
hint — which Claude then withdrew, since a guardrail `if` inside the
loop is already the language's way to state a fact Precog reads; (c)
none — trip-by-trip and widening only (Kespar). Owner asked what
guessing changes: how many ordinary loops the final build refuses.
Owner: "a and fallback to c" — templates first; no fit or no
confirmation → unproven, as if guessing did not exist. The template
list is provisional, grown by measurement. `design/precog.md`.

### 2026-09-22 — `design/precog.md` opened, ported from Kespar (owner: "Let's go")

Kespar's three-layer method carried over as the starting point; every
number Kespar marked provisional stays provisional and is listed under
Open; what Kespar lacked (ownership as the parallel proof, failures as
domains, range types and bounded lists as domains, the final build,
LLVM range metadata, content-addressed reuse, the 32 GB / scaling
requirement) added from this repo's decisions. Questions follow one at
a time, starting with loop invariants.

### 2026-09-21 — Precog has a first design: Kespar (`~/Kespar`, 2026-09-16..19)

Not a decision; a correction of the record. Claude had said all day that
Precog was undesigned. The owner: "Kespar was built by your Precog docs."
`~/Kespar/design/precog.md` (149 lines) + ~14.6k lines of Rust: a
three-layer Precog — known values executed at compile time; bounded
values tracked as exact sets (≤4096) then intervals, loops to a fixpoint
with widening and trip-by-trip following (100k/loop, 2M/program),
functions inlined per call site (depth 8); what is left asked of Z3 as a
bit-precise query (unroll 16/64, `rlimit` not timeout) — plus a
separately-authored oracle, a program generator that found Precog bugs,
and a bench (sieve 14.7 s → 2.6 s of compile time after four engineering
fixes). No clock anywhere: same answer on every machine. Missing by its
own account: loop invariants. The POC is finished ("abandoned"); the
Cyborg Precog document starts from it. Its provisional numbers are open
questions here.

### 2026-09-21 — v1 public release includes GPU support (owner, direction)

Stated while discussing hardware and cross-compilation. Not designed.
Three readings, cost ascending: (a) GPU libraries through `extern`
(already possible in principle; a bindings Limb); (b) kernels written in
Cyborg, compiled via LLVM to PTX / SPIR-V / Metal, Precog proving the
index math; (c) automatic offload of a parallel `for`, the parallelism
pillar extended to the GPU, dependent on (b). Owner: **c**. So: the
programmer writes `for`, Precog decides CPU cores or GPU, `for seq` says
no — no GPU syntax in the language. (b) exists underneath as mechanism
(loop bodies compiled via LLVM to PTX / SPIR-V / Metal, a runtime that
launches and copies). Precog's offload test: iterations provably
independent (the existing parallel test), trip count worth the bus
transfer, body within what the GPU can do (no text, no `extern`, no
failure escaping the loop). Undesigned beyond this.

### 2026-09-21 — Three oracles: interpreter, Precog, translation validation (owner)

Owner asked how many oracles there should be. Claude: one interpreter
(the triangle docs / oracle / compiler closes), and if a third existed
it should be a different *kind* of check — a brute-force Precog oracle
(the one worth having), translation validation, or the program
generator (already decided). Owner: "Interpreter + Precog +
translation-validation seems good." Recorded as the intended set in
`design/compiler-verification.md`. Added the same day (owner): **twin
programs** — other languages as oracles for the parts they share. Benchmarks the same day: the oracle
(release) runs the 1000-digit pi spigot in 49 s vs CPython 0.49 s vs
C++/Rust 0.023 s — as intended; it is the spec, not the product.

### 2026-09-21 — Cold read 5 (`examples/inventory.cpls`): nothing changed; a wrong recommendation caught

A fresh chat traced the output correctly, praised the range types
("build the language around it"), failures in the signature, `bin64`,
and quoted names for keyword-proofing. It asked about `otherwise fail`
(passes the name on — decided), `std::` vs `list.`/`char.` (language ops
vs the Limb — decided, docs), unhandled `list.append`/`remove` (check
sites, the tiers — decided), `std::exit` in an arm (arms must leave —
decided), `>==`, brackets, 1-based, quotes (reader five). It called
`let mut 'best'` beside an unmarked owned parameter inconsistent; Claude
accepted that Rust-shaped assumption, recommended `mut` on parameters
and argued against "auto mut" — then checked: plain `let` and plain
parameters are Precog-inferred (2026-09-14), the oracle runs
`let 'x' := 1; 'x' = 2;`. Nothing inconsistent, nothing changed; a docs
line added. Lesson recorded in memory: verify in DECISIONS/design and
the oracle before recommending. Also noted: the example's
`heaviest[...copy 'shelf']` teaches an expensive pattern — the example's
fault, not the language's.

### 2026-09-21 — Extern data: `extern let ['mut'] 'x': T;` (owner)

Open since the FFI decisions. Proposed: `extern let` mirroring `extern
fn` (reads and writes nocheck sites, C-shaped type, no value); no extern
data, only C helper functions; `extern` on a `ptr` read with `ptr.read`.
Owner: a. Noted: `errno` is a macro on macOS/Linux, so std will offer
`std::c.errno[]`; `extern let` is for plain globals.
`examples/extern_data.cpls`, five reject tests.

### 2026-09-21 — `char` conversions: `char.code` / `from_code`, text → char fails, units down fail (owner)

Three questions. (1) Number ↔ char: `to.int32[`7`]` gives the code (55,
the C trap), the digit (7, a parse in disguise), or is refused with two
named operations — `char.code[…]` and `char.<unit>.from_code[…]
otherwise …` (fails `not_a_char`), digit value via text — owner: c. (2)
Text → char: a failing conversion `to.char.point["A"] otherwise …`
(`not_one_char`), or only `std::str.at` — owner: a. (3) Between units:
down fails (`not_one_point`, `not_one_byte`), up always works, or all
refused — owner: a. Oracle fix on the way: a char literal now takes its
declared unit (`let 'g': char.grapheme = `é`` was staying a point).
`examples/chars.cpls`, four reject tests.

### 2026-09-21 — A module and a type cannot share a name (owner)

Proposed: refuse (one name, one thing, per Limb) or allow and tell apart
by what follows the dot. Owner: a, refuse. Covers module blocks and file
modules, against structs, enums and interfaces. Reject test
`module_type_clash`.

### 2026-09-21 — LLVM through `inkwell`, then `llvm-sys`, then the C++ wrapper; version follows inkwell (owner)

Owner asked how the language is built and in what (recap of the
2026-09-14 pipeline: Rust front end, checker + Precog, lowering via the
LLVM C API, LLVM, system linker; `no_std` Rust runtime; std in
CyborgPL; not self-hosted), then inkwell vs llvm-sys: safety difference
is in the compiler's own code only; inkwell 0.10 is on LLVM 22.1 while
LLVM 23.1.1 and llvm-sys 231 are out; the drop-down to llvm-sys reaches
unwrapped C-API calls on the *same* LLVM, not a newer one. The 22 → 23
changes that touch Cyborg: branch-opcode C-API change (inkwell's), IR
byte type (unused), AArch64 clobber bug for `x29`/`x30` written as `xN`
— dodged by renaming to `fp`/`lr` in the asm lowering, as Clang does.
Owner: record it. `design/implementation-languages.md`, `design/ffi.md`.

### 2026-09-21 — bin → int fails unless whole: `not_whole` (owner)

Open since 2026-09-19. Proposed: fail unless whole (rounding said first,
`to.int32[std::round['x']]`), truncate (C, quiet), floor (matches `/`,
quiet). Owner: a. The rounding toolbox — `std::round`, `floor`, `ceil`,
`trunc`, a bin to a whole bin — is std's; names provisional.
`examples/whole.cpls`.

### 2026-09-21 — `list.remove` hands the element back; no `give` of an element (owner)

Surfaced while explaining `...give`: the oracle had refused
`give 'xs'[pos i]` since functions-as-values ("it would leave a hole")
without the owner ever being asked — Claude's default, now put properly.
Owner asked what giving an element is for (moving a big owned element
into a function without a copy), why no hole (every list access in the
program would carry an "is it still there?" check; a marker that keeps
positions is an `enum` element's job), whether shrinking does what a
hole can (all but keeping positions), and whether a `give` that shrinks
differs from a `list.remove` that hands back (no — two spellings).
Owner: `list.remove` hands. Bare `list.remove[…];` still just drops it.
`list.append` as a value is refused. `examples/queue.cpls`.

### 2026-09-21 — Owned elements through `...`: say the verb — `...copy 'xs'` / `...give 'xs'` (owner)

Proposed: always copy (safe, a copy per element), always give (moves
out, empties the list), or say it after `...`, as every owned hand-over
in Cyborg says its verb. Owner: c. A `give` empties the whole list —
never holes. No verb into a `lent ...` lends each element; plain
elements take no verb. `examples/spread.cpls`.

### 2026-09-21 — `...'xs'` unpacks a list into a variadic call; only where its length is known (owner)

Open since 2026-09-19. Owner first asked what the line even does
("unpacks the list: `sum[...'xs']` becomes `sum[1, 2, 3]`"), confirmed
`sum` is an ordinary variadic function, then said yes. Follow-up: a list
whose length Precog cannot see (from input, no bound) — (b) a compile
error at that call, "give it a bound or pass it as a list", or (c) the
compiler quietly builds a list-taking version. Owner: b. Forwarding a
bundle to another variadic is the same mechanism (a bundle's count is
always known) and closes that older open item. Unpacking owned elements
into an owning variadic (copy or give?) is left open; the oracle refuses
it. `examples/spread.cpls`.

### 2026-09-21 — A length bound on a list: `list[T][lo, hi]` (owner)

Opened from the `uint64` decision: how to tell Precog a length it can't
see. Three placements proposed — `list[int32][0, 1000]` (the thing, then
its bounds, as `int32[0, 100]`), `list[int32, 0, 1000]` (mistakable for
three things), `list[int32] length[0, 1000]` (a word). Owner: `a`. Claude
applied the existing rule for what follows — every change to a bounded
list is a check site, its length is `uint64[lo, hi]`, low bound ≥ 0, no
bound on `array` — and asked the owner to confirm. `examples/bounded.cpls`.

### 2026-09-21 — `list.length` / `array.length` are `uint64` (owner)

Open since 2026-09-17: the number's type. Proposed `int32` (matches
counters; lies past 2 billion), `uint64` (a count: never negative, never
too big; narrowing into `int32` is a check site Precog proves wherever it
can see the list) and `int64`. Owner asked whether Precog makes the
choice moot — it makes the check free in the visible case, and the final
build may store a proven-small length however it likes; the written type
is for the reader and the unseen case. Owner: "`uint64` it is". For the
unseen case (a list from input) the owner asked for a way to state the
bound: a guard `if` already works; a length bound on the list's type
(`list[int32][0, 1000]`) is opened as the next question.

### 2026-09-21 — `&` / `&mut` as alternatives to `lend` / `lend.mut` (and `lent` / `lent.mut`); `printu` kept, long alias added (owner)

Four cold readers in a row tripped on `lent` vs `lend` and called `printu`
cryptic. Claude laid out the cost of each (one letter apart; a made-up
word) and asked what, if anything, to change. Owner: keep `printu`, add
`std::this_will_print_the_useful_numbers.this_will_go_to_the_standard_output`
as its alternative (the `…standard_error` twin follows); add `&` / `&mut`
as alternatives to the lending verbs. Both spellings are equal — `&'s'`
is `lend 's'`, `&mut 's'` is `lend.mut 's'`, `&'x': T` in a parameter is
`lent 'x': T`, `fn[&T]` is `fn[lent T]`. Binary `&` between two values is
still an undefined custom operator. Every `&` / `&mut` makes the build
print `What are you? Rust programmer or Claude?` on stdout — one line
per `&` — then carry on (owner: "You're not supposed to use that"). `examples/alternatives.cpls`.

### 2026-09-21 — Cold read 4 (`examples/shop.cpls`): lends may escape (kept), three notes (owner)

A fresh Claude chat read the shop program: the call-site verbs are "the
thing to build around", the failure system and `pos` earn their place;
it asked whether a borrow can escape a call, what arithmetic does to a
range type, how failures propagate, where `<T>` is introduced, whether
interfaces are structural, and why mutation is unmarked — all decided,
mostly not visible in one program. It misread `int32[0, 100_000]` as a
generic instantiation (a cost, logged), claimed there is no char literal
(there is: backticks), and called the quotes "two shifted keystrokes" —
they are unshifted (owner). Recurring for the fourth reader: `lent`/`lend`
one letter apart; `printu` cryptic.

Claude proposed adopting "a lend never outlives the scope that made it"
(no lifetime annotations ever; a returned view becomes a copy Precog
elides where it can) and, asked why it was trying to change it, admitted
the reason was that the oracle already behaves that way. Owner: "Keep it
as it is." So: views may escape, Precog infers lifetimes, the optional
annotation's spelling stays open, and **the oracle's refusal to return a
lend is a known gap, not the design.**

### 2026-09-21 — Callbacks: capture-free, C-shaped functions cross to C; `fn[…]` in extern signatures (owner)

Owner asked what callbacks are for (sorting, events, signals, servers).
(1) what may cross: only capture-free functions (a), over smuggling
captures via userdata (b) — owner asked what (b) was for, chose a.
(2) `fn[…]` in an extern signature, C-shaped, by value, no fail; a C
function pointer received is a `fn[…]` value called under `nocheck` —
yes. Consequence: `ptr[T]` allowed in any fn signature. Oracle emulates
`qsort` with the callback (`examples/callbacks.cpls`), six reject tests.

### 2026-09-21 — Functions as values: bare name, `fn[…] -> T` types, `with [lend 'v']` captures (owner)

Owner asked what it is for (sorting by a rule, callbacks, map/filter, C
callbacks). (1) a bare name is the function as a value, `'f'[…]` calls it
— yes. (2) the type is the signature without the names, verbs included —
owner asked how it compares to a normal fn (names and body removed) —
yes. (3) named-only / inline-without-capture / inline with explicit
`with [lend 'limit']` captures — owner: c. `design/functions-as-values.md`;
oracle: `examples/functions_as_values.cpls`, ten reject tests; `lend
'xs'[pos i]` added on the way. Closes open item 11.

### 2026-09-21 — Bitwise: `and.bit`, `or.bit`, `not.bit`, `xor`, `shl`, `shr` (owner)

Owner asked why the spelling extends from `and`/`or`/`not`, then what
"keep a 1 only if both have a 1" meant, then how that relates to `and`
— walked through column by column; "`and.bit` it is." No precedence
(parenthesise), integers only, shifts are check sites.
`examples/bitwise.cpls`; the FNV-1a hash example now uses `xor` and
matches the real algorithm (1335831723 for "hello"); six reject tests.

### 2026-09-20 — Input: `std::input.line` / `.all` failing with `closed`; range types `int32[0, 100]` (owner)

(1) reading: two std functions returning `str.utf8`, `closed` on nothing
left — yes (owner caught Claude's nested `otherwise` inside a bracket,
which the whole-right-hand-side rule forbids; two lines instead).
(2) how Precog learns an outside value's range: guardrail `if` / bound on
the conversion / range types; owner noted `int32` is already a bound;
chose **c, range types**. (3) spelling `int32[0, 100]` — yes. (4) two
ways in: assignment (a check site) and `to.int32[0, 100][…]` (fails
`out_of_range`) — yes. `design/input.md`; oracle: `examples/input.cpls`,
`examples/ranges.cpls`, ten reject tests. Closes "input" and "boundary
range annotations".

### 2026-09-20 — Wrapping arithmetic: `+~ -~ x~ xx~` (owner)

Homes: std functions / operators / a wrapping type; Claude recommended
operators (the floor-vs-`div.trunc` logic: the default operator is the
safe meaning, the other has its own spelling) with Zig's `+%`; owner:
"`%` is percent, not modulo" — Claude offered Swift's `&+` or a tilde;
owner: "`&` reads like borrow" — "`~` it is." `examples/wrapping.cpls`
(FNV-1a with no `nocheck`), four reject tests. Surfaced: no bitwise
operators (xor, and, or, shifts) exist yet — logged open.
Owner then floated `and or not` → `& / !` and `/` → `div`; Claude: `div`
and `!` fine, `&` has the borrow reading the owner had just rejected, `/`
for or would be misread by everyone; offered two coherent sets. Owner:
"Nvm, back to normal." Unchanged.

### 2026-09-20 — Interfaces: `interface`, operators definable for user types, automatic satisfaction + `satisfies` claim, `<T: Ordered>` (owner)

"How do you say what your generic supports?" — you don't yet; owner:
"now". (1) may a type define the language's operators for itself — yes
(revises the 2026-09-19 no-redefinition rule: never for the language's
types). (2) `interface Name { fn …;  op …; }` about some `<T>`, and
`<T: Ordered>` on one mention — yes. (3) automatic / declared / both:
owner asked twice what the options meant, chose (c) automatic plus an
optional checked claim; asked what `is`, what `Ordered`, what the claim
line are; worried `is` reads wrong with other names — Claude offered
`satisfies`; "`satisfies` it is." Consequence set by Claude: verbs on a
`<T>` value are allowed and do nothing for a plain type, so generic code
can `return give 'a'`. `design/interfaces.md`; oracle:
`examples/interfaces.cpls`, nine reject tests. Closes "generic
constraints".

### 2026-09-20 — Generics: `<T>` on every mention; struct/enum blanks by first appearance (owner)

Claude offered a declared list (`fn max [type T] […]`) vs implicit; owner
proposed `fn max ['a': <T>, 'b': <T>] -> T`, then "both use `<>`" — every
mention. For structs, Claude proposed a bracket after the name for order
(`struct Pair [<A>, <B>]`); owner asked why, then "how's it without
generics?", then "why does it need `[<T>]`?" — it doesn't; owner: "no
bracket, first appearance." `design/generics.md`; oracle:
`examples/generics.cpls`, eight reject tests. Closes the generics
spelling item; constraints are open.

### 2026-09-20 — Custom operators: `op`, words and glyphs, any run, no precedence, lent-or-`give` operands (owner)

Owner: "How do you make a custom operator?" — not yet; then "one by one".
(1) words only vs glyphs too — owner: glyphs too. (2) the characters
`! % & * + - / < > = ? @ ^ | ~`; Claude proposed a three-character cap;
owner: `<+>-` is just an operator, a negative goes on the operand — a
maximal run is one operator, no cap. (3) precedence — owner: "Require"
(parentheses whenever a custom operator meets another). (4) operands —
Claude proposed always-lent; owner asked the difference, then how Rust
behaves (by value; four impls), then which has better performance
(by-value can reuse a dead operand's buffer); owner: "middle" — lent by
default, `give` on an operand takes it by value and the caller writes
`give`. `design/operators.md`; oracle: `examples/operators.cpls`, thirteen
reject tests. Closes the "custom operators" open item.

### 2026-09-20 — The borrowed prints are removed (owner)

"Remove `std::<lang>.<prints>`." `std::c.printf`, `std::c.fprintf.stderr`,
`std::rs.print` / `println` / `eprint` / `eprintln` — decided 2026-09-19 as
a joke feature, gone a day later. `print` and `printu` are the prints.
The oracle refuses the old names by name; examples and tests removed.

### 2026-09-20 — `/` floors, `mod` is the mathematician's; truncation is `std::div.trunc` / `std::mod.trunc` (owner)

Owner asked the difference (negatives only: `-7 mod 2` is `1` floored,
`-1` truncated — the clock/hash bug), chose floor, then "should we leave
hardware that to std?" — yes. Also confirmed ("yea") the final-build
understanding: working build keeps runtime checks at unproven sites, the
final build makes them compile errors, nothing unproven ships. Oracle
switched from its day-one provisional truncation; `examples/division.cpls`.

### 2026-09-20 — The final build has no runtime checks: an unproven site is a compile error there (owner's clarification of 2026-09-14)

Owner, on being told the unproven runtime checks ship: "Oh shit, that's
not what I meant" — "'runtime checks stay, no matter the cost' — that was
when building around, dev testing, not the final product." So: two
builds, the same full Precog, differing only in what an unproven site
becomes. Working build (`jetpack build`, proposed): a runtime check, so
the program runs and crashes loudly where a guardrail is missing. Final
build (`jetpack final`, proposed): a compile error with the
counterexample — the default tier behaves as `check`. Nothing unproven
and nothing runtime-checked ships; `nocheck` remains the only way to ship
an unproven site, listed in the build output. Not the rejected
development mode: nothing is skipped, the final build is stricter. The
2026-09-14 tiers entry is corrected below rather than appended to.
Claude's understanding was restated as a table and awaits the owner's
"yes"; the build-command names are Claude's proposal, JetPack's, so no
file-wide switch enters the language.

Owner confirmed the next day ("yea"); `/` and `mod` decided then too.

### 2026-09-20 — Enums: `enum Status { empty  claimed: str.utf8  locked }`, `Status.claimed["x"]`, `match` (owner)

Owner asked "didn't we already do enums?" (no — failures chose inline
lists, leaving item 4 open) and "what are enums?" (one of a few options,
like `bool` with your own names). Declaration shape and `enum` keyword —
yes. Branching: the `otherwise` arms with a word in front; owner asked
for a small program, then "why `match`?" — the value is matched against
shapes; `switch` has fall-through baggage, `when` reads as time. "`match`
it is." Consequences set by Claude: `match` is a statement; an enum is
always owned like a struct; `==` works. `design/enums.md`; oracle:
`examples/enums.cpls`, thirteen reject tests. Closes open item 4.

### 2026-09-20 — Text: three counts, no length; walk by unit; no `[pos]`; toolbox in std; exact `==`; no `<` (owner)

Four questions. (1) length: no `str.length` — `str.bytes` / `str.points`
/ `str.graphemes` (owner asked "what?", got the emoji example, yes).
(2) walking: `str.each.<unit>[…]`, no bare `for` over a text — yes.
(3) the toolbox (`split`, `trim`, `upper`, …) is std's, `==` compares
bytes exactly, `<`/`>` on text is an error — yes. (4) no `'s'[pos 3]`;
`std::str.at.<unit>` with a failure — yes. `design/strings.md`; oracle
enforces the language half and emulates the std half
(`examples/text.cpls`, seven reject tests). A `char` now compares by
content across units (Claude, needed for the example).

### 2026-09-20 — Inline assembly: a statement, raw braces, arrows for direction (owner)

Three questions: statement vs value — owner: statement (option 1); the
block — Claude first wrote a list of strings (Rust's shape), owner: "I
don't think Assembly looks like that" — raw verbatim braces; bindings —
Claude proposed words and arrows both, owner asked what the arrows were
for, then: "drop the words, keep the arrows." Then the head: owner
proposed `nocheck inline_code.ASM`; Claude offered house style
`inline.asm` (no underscore, lowercase) and the `inline.<language>` family
(`inline.C` later via clang); owner took `inline.` but kept the capitals —
"the inline code name should be easy to spot, just like `MAIN`" — so
**`nocheck inline.ASM { … } with ['v' -> x0, 'n' <- x0, clobber x16];`**.
In `design/ffi.md`; oracle parses/checks and refuses to run.

### 2026-09-20 — FFI in the oracle (Claude, per yesterday's decisions)

`extern fn`, `ptr[T]`, `nocheck` on extern calls and pointer reads/writes,
`ptr.is_null`, `layout.c struct` with C-shaped fields, variadic externs,
`size.of`/`align.of` — all enforced; libc emulated for six functions so
`examples/ffi.cpls` runs. Nineteen reject tests. `asm` refused as
unspelled. Surfaced: a `str` has no length operation yet (string
operations, open).

### 2026-09-19 — Low level is the base language's: `extern`, `ptr[T]`, `layout.c`, `size.of`, inline asm (owner)

Owner asked how to print without std, then how std's print gets made
(intrinsics → runtime → syscall), then — "I don't wanna rush (like Xag)"
— asked what `extern` is; Claude explained from "a program is a pile of
named functions" up. Then one by one: the OS door is open to every Limb,
not std-only ("Syscall and lower-level stuff should be able to do, using
the base Cyborg language"); asm is in too. (1) `extern fn name […] -> T;`
— yes. (2) pointers: `ptr[T]` only in extern signatures, a lend satisfies
it — owner: a. (3) an extern call is a `nocheck` site — "yes, reuse
`nocheck`". (4) a returned pointer: hand back only, or read through —
owner: **b** (read/write through), against Claude's recommendation of a.
(5) `ptr.read/write/at`, `nocheck` sites — yes. Missing features listed;
`ptr.is_null` — yes; `layout.c struct` — yes; variadic externs — yes;
`size.of`/`align.of` — yes (owner asked what size and alignment are).
Raw allocation: none in the language (`list[uint8]`, or `malloc` as an
extern). `design/ffi.md`. Not yet in the oracle. Open: asm spelling,
callbacks, extern data.

### 2026-09-19 — `std::uenr` renamed `std::uenr`, Unbounded Exact Numerical Representation (owner)

Rename only; semantics unchanged. Owner also set the focus: the base
language before std.

### 2026-09-19 — Variadic parameters: `...'name'`, compiled per call-site shape (owner)

Model: (a) a list (needs a list-of-anything) vs (b) a fixed set per call
site, specialised by Precog; owner asked "wouldn't we eventually need
generics?" — (b) is the same machinery; owner: b. Spelling: Claude offered
`many`; owner: could be confused with `array`; Claude offered `...` /
`vararg` / `rest`, recommended `...` (the ellipsis, universal); owner
asked why; "`...` it is." Owner asked about two types (`: int32, bin16`)
— no: untyped keeps each argument's own type, the body constrains; a
union type is not a thing yet. Oracle: `examples/variadic.cpls`, four
reject tests. Closes the "variadic parameters" open item.

### 2026-09-19 — Borrowed prints live under their language: `std::c.printf`, `std::rs.println` (owner) — removed 2026-09-20

"`std::<lang, only if it isn't Cyborg>.<that lang's print(s)>`" — so
`printc` → `std::c.printf` / `std::c.fprintf.stderr`; `printrs` →
`std::rs.print` / `println` / `eprint` / `eprintln` (Rust's own names, and
`println` adds the newline as Rust's does). Owner also asked whether the
values inside could go unquoted; Claude: it would need variadic
parameters (undecided, and needed for std's own `print`) and a
per-parameter `bare` flag, which makes call parsing depend on the callee
and adds a second dialect; recommended variadics soon, `bare` shelved.
Owner: "yup." Open: variadic parameters; `bare` (advised against).

### 2026-09-19 — `std::printrs`: Rust's `print!`, exactly, with checked holes (owner) — removed 2026-09-20

"Now, `printrs`, exactly like Rust's." `{}`, `{0}`, `{name}` from scope,
`{:?}`, fill/align/sign/#/0/width/precision/type, `{{` `}}`, `\n` inside
the format. Oracle: `examples/printrs.cpls`, five reject tests. Owner then
asked whether `printc`/`printrs` could be called with `( )` (a Limb-granted
ability); Claude advised against — two call spellings, and `( )` is
arithmetic grouping only by design; owner: "stick with `[]`, at least the
insides look alike." Closed.

### 2026-09-19 — `std::printc`: C's `printf`, exactly, with checked holes (owner) — removed 2026-09-20

Owner asked what C's print looks like and what it would look like here,
then "I kinda feel like adding `printc`"; Claude proposed a checked subset
with the newline as a trailing value; owner: "`printc` is exactly like
C's" — so `\n` inside the format is `printc`'s to interpret, the base
string rule untouched. Precog checks holes against values (a mismatch is
a compile error). Oracle implements d i u x X o f e g s c b %%, flags,
width, precision; `examples/printc.cpls`, five reject tests. The base
language has no print at all (owner asked; it is std's, a boundary).

### 2026-09-19 — Strings have no escapes but `\"` and `\\`; the newline is the bare `\n` outside (owner)

Owner asked whether `"\n"` prints literally — the oracle's lexer had
C-style escapes by default. Owner: "That's why I made it outside" — the
bare `\n` piece is the newline; then: "escape in double quotes uses `\`,
but only for what needs that when inside" — `\"` and `\\`. Anything else
after a backslash is literal. `examples/strings.cpls`.

### 2026-09-19 — A keyword may name a function, reached by path only (owner)

Owner: "`std::to` wouldn't be ambiguous, even with `to` being the lang's,
right? it has `std::`." Claude: correct from outside; inside its own Limb a
bare `to[…]` would collide, so the rule is "reached through a path, never
bare" — `std::to[…]` outside, `limb.to[…]` inside. Owner proposed `std:to`
then `std:::to` for the inside; Claude: `:` is taken and `limb.` already
means this. Owner: "Yup." Functions only. Oracle enforces
(`examples/keyword_fn.cpls`, `keyword_fn_bare` reject).

### 2026-09-19 — Explicit conversion: `to.int32[…]`, `to.str.utf8[…]`; `++` takes text on both sides (owner)

Cold read #3 flagged `++` converting a number implicitly — the second
reader to do so. Claude offered (a) keep it or (b) make it explicit with a
types-as-calls spelling (`int32["42"]`); owner asked what
`int32["42"] otherwise 0` was, then why not `std::to.int32`; Claude: the
compiler owns conversion, and text→number failing uses the failure model.
Owner: "b, `to.int32[…]`?" — a bare keyword head, like `list.length`.
Consequences set by Claude: `++` demands text on both sides; text→number
fails with `not_a_number`; narrowing is a check site; encoding always
spelled; stored form. `design/conversion.md`; oracle enforces (nine reject
tests); every example converted. Closes open item "explicit conversion
spelling"; `std::exit[…]` is also now a leaving statement after
`otherwise` (cold-read consequence).

### 2026-09-19 — Failures: a value the caller must handle; `-> T or fail[…]`, `fail name;`, `otherwise` (owner)

Owner asked what the base language needs; Claude ranked error handling
first. Five questions: (1) model — exceptions / failure-as-value /
sentinel; owner: **failure as a value** (b). (2) what it carries — nothing
/ a name / a name and a payload; owner: **name and payload** (c). (3) where
the reasons are declared — inline per function / a named `enum`; owner
asked what an enum is, then: **inline** (option 1), so no enum yet. (4)
the function's side: `-> bool or fail[full, bad_item: str.utf8]`,
`fail full;`, `fail bad_item["x"];` — owner: yes. (5) the caller's side:
a fallback value, a leaving statement (owner asked "isn't it `return
false`?" — both exist), `fail` to pass up, or arms with every reason
covered; owner: yes. Claude then caught that `or` collides with the
boolean operator; candidates `else`/`catch`/`otherwise`/`onfail`; owner:
**`otherwise`**. Full text in `design/failures.md`; oracle enforces
(`examples/failures.cpls`, 18 reject tests).

### 2026-09-19 — Text joins with `++`; the whitespace join is gone; `print` takes many values (owner)

Owner: "Feeling like adding `++` for strings connecting." Claude asked
whether it coexists with the whitespace join or replaces it; owner asked
"is our current method implicit?" — yes, the space was the operator, and
the number→text conversion was implicit too; owner: "we'll make it
explicit." Glyph shortlist (`++`, `..`, `...`, `<>`, `&`, `~`, a word such
as legacy's `stch`); Claude recommended `++`; owner: "`++` it is."
Consequences Claude set and flagged: precedence below `+ -`, above
comparisons, left-assoc; at least one side text, the other rendered in
stored form (the 2026-09-14 "number piece → stored form" rule survives,
the "bare number is never a piece" rule is retired since nothing is
ambiguous now); two values side by side are an error; `print`/`printu`
take any number of comma-separated values and render each — the only way
`printu` can still show the useful form of a number, since a `++` join has
already fixed the text. Oracle, examples (`examples/join.cpls`), tests,
docs converted. The "whitespace: separator or implicit space?" question
closes with it.

### 2026-09-19 — Field names are bare: `struct Locker { number: int32 }`, `Locker[number = 7]`, `'l'.number` (owner)

A cold read (a fresh Claude chat shown `examples/lockers.cpls`) caught the
inconsistency: fields were quoted in the declaration and the construction
but bare at access. Claude offered (a) quote everywhere (`'l'.'number'`)
or (b) bare everywhere, and recommended (b); owner asked "is `number` a
var?" — no, a part of one: the quotes mark the thing you hold (`'l'`), the
route into it (`.number`, `[pos 2]`) is bare. Owner: "b". The one-line
rule is now: *quotes mark a value you can hold — a variable or a const;
names reached through a dot or a Limb — functions, modules, fields — are
bare.* Oracle, examples, tests converted; the quoted spelling is refused
with a message.

### 2026-09-19 — The ladder is default → `check`; `nocheck` is the exit, not the last rung (owner)

Owner's original picture: build guardrails, then write `nocheck` to remove
the check. Clarified against the 2026-09-14 tiers entry ("proves what it
can and inserts a runtime check wherever it can't"): once a guardrail is
good, Precog removes the check by itself, with or without `check`;
`check` only adds "refuse to compile and show the counterexample if it is
not enough"; `nocheck` is for the site no guardrail can make provable.
Owner: "if that's better, then yes. Not the original one." A proven check
can never fire, so keeping it is pure runtime cost. Build output lists
`nocheck` sites **and passed `check`s** (owner, 2026-09-19). Claude also
proposed listing default-tier sites still carrying a runtime check ("where
to add a guardrail") — not yet answered. **No alias for `nocheck`** (owner,
2026-09-19): a `*` shorthand was raised for codebases dense with it; Claude
advised that `nocheck` should stay long and rare (Rust's `unsafe` is ugly
on purpose) and that `nocheck { … }` covers the honest dense case; owner
closed it.

### 2026-09-19 — `check` / `nocheck` spelled: a word in front of a statement (owner)

Owner asked for the tiers again, confirmed they are never global, asked
for a syntax; Claude proposed one rule — the word before a statement,
covering its check sites; a `{ }` block is a statement so `check { … }`
needs no second rule; innermost wins; never on an expression, a `fn`, or
the top level — and walked through a guardrail as a plain `if` before the
site. Owner: "So, `check` & `nocheck` at the front?" — yes. Oracle enforces
(`examples/check_tiers.cpls`, seven reject tests); the oracle cannot prove,
so a firing `check` reports the counterexample value and a firing
`nocheck` reports the lie. Only `nocheck` sites are listed in build output
(Claude's whichever, flagged).

### 2026-09-19 — Modules (owner)

Owner asked what Rust's `pub()` is, then what modules are, then "Yeah, we
should have modules." Five questions, one at a time:
1. File-as-module (a) vs `module` block (b) vs both (c): owner asked why not
   (b), what (a) misses, what (c) is — chose **(c)**: every `.cpls` is a
   module named after it (directories nest) and `module name { … }` nests
   a block. Keyword **`module`** — owner caught that `mod` is the remainder
   operator.
2. Root: **`limb.cpls`** (owner chose (a) over "no root"; asked about the
   `.limb` archive and kept the name).
3. Names: **"whatever has most compat"** — lowercase ASCII identifier, no
   Windows device names; a bad file name is a compile error.
4. Middle audience: **`pub.limb`** (owner chose the chain over `pub(limb)`,
   which would break "`( )` is arithmetic grouping only", and over no
   middle level).
5. Reaching the root from a module: **`limb.helper[…]`** (owner chose (c)
   over lexical fallback and over the Limb's own name).
Consequences written into `design/limbs.md`: bare = this module, dotted =
relative, `limb.` = absolute; visibility checked along the path; file
modules provisionally public. Oracle runs Limb directories
(`examples/geometry/`), blocks (`examples/modules.cpls`), 20 reject tests.

### 2026-09-19 — No public variable (owner)

Asked twice how to make a public variable; Claude: none exists, and laid
out what one would be for (configuration knobs, shared counters/caches,
singletons) and the owned shape of each (parameter or struct field; a
value lent mutably; what `MAIN` owns). Owner: "No public variable." A Limb
exposes `pub const`, `pub fn`, `pub struct` only. Closed. In
`design/limbs.md`.

### 2026-09-19 — `pub`; a Limb's top-level value is `pub const 'name'` (owner)

Owner asked how to declare a public variable. Claude: a Limb has no
top-level variables yet; proposed `pub` before `fn`/`struct`/`let` with a
top-level `let` immutable only. Owner asked what top-level variables are
for; Claude: named constants (legitimate), configuration (a boundary
value), mutable program state (conflicts with ownership). Owner: "So, it's
`pub const`?" — yes; and the name is **single quoted** (`'pi'`,
`mathstuff::'pi'`), owner's choice over a bare function-like name. Rules
in `design/limbs.md`; oracle enforces (`examples/consts.cpls`, twelve
reject tests). The author's "reachable bare" flag stays open.

### 2026-09-19 — No `import`/`use`; the Limb maker decides bare reachability (owner) — bare dropped 2026-09-23

Asked the difference between `import` and Rust's `use`; Claude: `import`
loads and binds, `use` only shortens a path that already works — and Cyborg
already sits on Rust's side (JetPack.toml lists, `::` reaches). Owner:
whether a Limb's items may be used bare is **the Limb maker's decision**,
stated in the Limb's source; a user cannot alias or override — "if you want
to override that, you edit the source code yourself." Spelling of the
author's choice is open, alongside `pub`. In `design/limbs.md`.

### 2026-09-19 — Syntax revision: `let`/`fn`, type after the name, `[ ]` only where it builds, bare control flow (owner)

The owner opened the day with "I'm thinking of changing the syntax to
Rust-style: `let 'x' := [1000]; let 'x':bin16 = [1000];`" and took it
through ten questions, one at a time. Everything not listed here stands
as decided before (quoted names, `:=`/`=`, the verbs, `::`/`.`, `[ ]`
semantics, operators, `MAIN`). Full spelling in `design/syntax-sketch.md`,
`functions.md`, `control-flow.md`, `structs.md`, `ownership-syntax.md`.

1. **`let`**, not `var` (owner). Claude had recommended keeping `var` —
   "says what it is"; the owner's own 2026-09-17 objection to `let`
   (`let.mut.bin16`) dissolves once the type leaves the keyword chain, and
   the quotes already mark variables.
2. **The type follows the name after `:`** — `let 'x': bin16 = 1000;` (owner's
   proposal, Claude recommended it). Touches only the five name-to-type
   sites: variables, parameters, return type, struct fields, loop counters.
   Types themselves keep their dot chains (`str.utf8`, `list.int32` — bracketed later that day,
   `std::uenr`).
3. **`mut` / `immut` before the name**: `let mut 'x': bin16 = 1000;`,
   struct fields `immut number: int32` (owner chose (a) of three; Claude
   recommended it).
4. **Parameters: verb before the name, owning parameter bare** —
   `fn count [lent 'text': str.utf8]`, `['a': int32]`, `[mut 'a': int32]`
   (owner chose (a) over Rust's type-position `'text': lent.str.utf8`, after
   asking how Rust does it; Claude recommended (a) for the def/call mirror).
5. **`fn`**, not `func` (owner, unprompted, with 4).
6. **Return type `-> T`** after the parameters: `fn add ['a': int32, 'b': int32] -> int32`
   (owner chose Rust's arrow over `: int32`; Claude recommended the arrow —
   `:` attaches a type to a name, and a result is not the function's type).
7. **`[ ]` only where it builds** (owner: "calm that `[]` spam down";
   Claude proposed the cut, owner accepted). Brackets stay for text joins,
   lists, call arguments, struct construction, index, `range[…]`; a single
   expression stands bare on the right of `:=`/`=`/`return`, in conditions,
   as a struct field's value. Pieces outside `[ ]` are an error. Supersedes
   the 2026-09-14 "mandatory in every value position" and "a stored call
   wears the outer bracket".
8. **`else if`**, **`while`**, **`for`** — bare (owner, three questions;
   Claude recommended each). `loop { }` stays.
9. **One `for`; `range[start, stop, step]` is what it runs over**;
   the counter is typed on its name (`'i': int32`); the per-bound `:=`/`=`
   markers are gone (owner asked why, accepted "it would say the same thing
   twice"). `range[…]` as a standalone value is undecided.
10. **Modifiers bare, before the name**: `for seq perm 'i' in range[1, 10]`
    — `seq` first, then `temp`/`perm` (owner chose (b) over `for.perm.seq`;
    Claude recommended (b)).
11. **`MAIN { }` stays** (owner; Claude recommended keeping it — its
    distinctness is the feature).
12. **`temp` is the default** (owner, later the same day): `for 'x' in …`
    is a temp loop; `perm` is stated when the variable outlives the loop;
    `temp` may still be written and changes nothing. Supersedes the
    2026-09-17 "temp/perm mandatory".

Also confirmed: a Limb-provided type is `'r': std::uenr`; style is a space
after `:`, none around `::`. `pub` was mentioned ("if we do `pub` too") and
is logged as open, not decided.

Oracle updated the same day (lexer `:` / `->`; parser rewritten for the
new grammar; every example and reject test converted; old spellings —
`var`, `func`, `loop.while`, `else.if`, `loop.temp.for`, `range[:= …]`,
`int32 'x'` — are refused with a message naming the new one).

### 2026-09-14 — The five core features (owner)

Proposed and chosen by the owner. Full text in [design/core-features.md](design/core-features.md).

1. **Everything at compile time** — every possible check runs at compile time; compile duration is explicitly not a concern.
2. **Ownership model** — one owner per value; compiler-determined freeing; no runtime cost.
3. **`:=` computes the *best* type** — not just any inferred type.
4. **Dot-chained syntax** — most syntax reads as a chain; "flow with the code".
5. **LLVM backend** — native code.

### 2026-09-14 — `:=` picks the type that is best for safety and performance (owner)

"Best" is defined as: the type that gives the safest *and* fastest final product, worked out by whole-program analysis of every value the binding ever holds. The compiler has effectively unlimited time, so it does the analysis rather than the programmer. Proposed options by Claude; the criterion chosen by the owner.

### 2026-09-14 — Unprovable `:=` bounds are a compile error (owner)

When the analysis cannot bound a value (it comes from outside the program —
input, files, the network), `:=` is refused and the programmer must write an
explicit type at that boundary. Downstream of the annotation everything is
provable again. No silent fallback to a wide/checked type. Options (a) wide
fallback, (b) refuse, (c) both proposed by Claude, who recommended (b); owner
chose (b).

### 2026-09-14 — Unproven safety properties get a compiler runtime check, removable only by a programmer guardrail (owner) — clarified 2026-09-19: a *proven* check is removed by Precog itself; corrected 2026-09-20: the runtime check is the *working build's* behaviour; in the final build an unproven site is a compile error

Where whole-program analysis cannot prove a safety property (overflow, bounds,
division by zero, …), the compiler **inserts a runtime check, regardless of
performance cost**. The check stays until the programmer (1) writes a real
guardrail — e.g. validates the input — and (2) explicitly disables that one
specific check via syntax. Example: a GUI field typed Bin16 receiving a larger
value must be caught by programmer-written validation, not by a compiler check
left in the final product.

Claude had recommended "compile error + programmer-supplied fact"; the owner
chose runtime checks as the default instead, with explicit per-site removal.
Note this does not alter the `:=` rule above: an unprovable `:=` is still a
compile error, since that is about choosing a type, not guarding a value.

### 2026-09-14 — Three check tiers at every safety-property site (owner)

Keyword spellings are placeholders; the tiers are decided.

- **default** (nothing written) — *"I have no guardrails yet, and I want you
  to help me."* The compiler proves what it can and inserts a runtime check
  wherever it can't. Never refuses to compile.
- **`check`** — *"I've built a guardrail, could you help me verify it?"* The
  compiler proves, exactly (bit-precise solver over the real program
  semantics, not an approximate domain), that the programmer's guardrail
  covers the property. Proven → the check is removed. Not proven → compile
  error naming the counterexample or the missing fact (e.g. an unbounded loop
  that needs a bound or invariant at the boundary).
- **`nocheck`** — *"Okay, I'm sure this is correct."* The check is removed
  with no verification. This is the only construct in the language that can
  make a final product unsafe, and that is the programmer's responsibility.

Claude proposed verified-only removal; the owner chose to also provide the
trusted tier, with the discipline left to the developer.

**Why the compiler needs the programmer at all (for the record):** a loop
whose iteration count depends on outside data has an unbounded effect; no
amount of compile time can bound it (Rice's theorem). The programmer supplies
the missing facts at the boundary — value ranges, length limits, loop bounds
or invariants — and the loop becomes provable.

### 2026-09-14 — Ownership with borrowing (owner)

Every value has exactly one owner; the owner's scope end frees it. Moves
transfer ownership; copies make an independent value; **borrows** give
temporary access without ownership, and the compiler proves the borrow never
outlives the owner and never aliases a mutation (Rust-style, with lifetimes).
Claude presented A (borrowing), B (moves/copies only with compiler-elided
copies), C (B plus opt-in verified borrows) and recommended B; owner chose A.

### 2026-09-14 — Lifetimes: inferred by default, optional verified annotation (owner)

The whole-program analyser sees every call site, so lifetimes are always
inferred and nothing has to be written. Lifetime syntax exists as an
*optional* annotation: writing one pins a function's borrow contract, and the
compiler verifies the body against it (same spirit as `check`). There is no
unverified form — a lifetime that isn't proven is a memory bug with no
performance upside. Claude presented infer-only vs both, recommended both;
owner chose both. Syntax deferred until dot-chaining is decided.

### 2026-09-14 — User names are quoted: `'name'` (owner; reaffirmed)

Every user-defined *variable* name is written in single quotes — `var 'x'`, `'x' + 'y'` — and a function's name is quoted at its definition (`func 'add'`)
— and every keyword is a bare dotted path. **This is settled; do not
relitigate.** It has been argued against (noise, unfamiliarity) and the owner
has reaffirmed it, because it is load-bearing:

- **No reserved words, ever.** `var 'var' := [1];` is legal. New keywords can
  never break existing code.
- **Word operators depend on it.** `x`, `xx`, `mod`, `and`, `or`, `not` are
  unambiguous only because a bare word is never a name.
- **Names can be anything** — `'+'`, spaces, emoji, `'is valid?'`.
- **Three token classes, visually distinct:** values wear `[ ]`, names wear
  `' '`, keywords wear nothing. A reader never needs the keyword list.
- Cost acknowledged: two extra (unshifted) characters per name, and a
  first-contact "why?" for readers from other languages. Accepted; the docs
  should state the reason up front.
- The "`'x'` looks like a character literal" objection is a prior from other
  languages; CyborgPL has no character type, so `'` has one meaning. Rule
  that follows: **if a character type is ever added, it must not use
  `'…'`.**

### 2026-09-14 — `[ ]` is the value bracket, and it is mandatory (owner) — superseded 2026-09-19: only where it builds

`[ ]` means "one value, built from what's inside". Text pieces join into a
text (`["line 1" \n "line 2"]`), a formula evaluates (`[10 + 20 + 30]`), a
single literal is still bracketed (`[10]`), and `std::print.stdout[...]` takes
the same construct. Every value position uses it — `var.bin16 'x' = [1000];`
— with no omission rule. Owner proposed bare-infix vs bracketed and chose
bracketed; Claude recommended the same.

### 2026-09-14 — Inside `[ ]`, a bare number is an operand, not a text piece (owner) — retired 2026-09-19 with the whitespace join; `"count: " ++ 10` is fine

Quoted literals, variable references, and specials like `\n` are *pieces*
and join as text. A bare numeric literal is an *operand* and must sit in an
operator expression. Two adjacent bare numbers (`[10 20]`) are a compile
error regardless of what precedes the bracket. Claude had proposed
"juxtaposition always joins as text"; the owner rejected it for bare numbers.

This is strict with no exceptions: `["count: " 10]` is also an error. A
literal belongs inside the string (`"count: 10"`); a computed number goes
into a variable first and is referenced as a piece (`["count: " 'n']`).
Owner, 2026-09-14.

### 2026-09-14 — Initial type family (owner)

`bin` (IEEE 754 binary float), `deci` (IEEE 754 decimal float), `int`
(signed), `uint` (unsigned), `str` (text), and — added once comparisons
existed — `bool`. Widths as suffixes in the chain
(`var.bin16`). Full table and notes in [design/types.md](design/types.md).

### 2026-09-14 — `str` is narrowed by encoding in the chain: `var.str.utf8` (owner) — now `'s': str.utf8`

Encoding is a chain segment on `str`, the same shape as a width on a numeric
type. Owner proposed the chain shape. Encodings `utf8`, `utf16`, `utf32`, `ascii`
(Claude's list, owner accepted); `:=` picks the best encoding from the
value's use (owner). Decided 2026-09-14.

### 2026-09-14 — `char` exists, narrowed by unit: `var.char.point` / `.grapheme` / `.byte` (owner) — now `'c': char.point`

A character type, with the chain saying which Unicode unit "one character"
means: `byte` (1 byte), `point` (one code point, 4 bytes flat), `grapheme`
(one grapheme cluster, variable size, owned). Claude laid out code point vs
grapheme cluster and proposed the two-unit chain; owner chose it and added
`byte`.

Character literals use **backticks**: `` var 'c' := [`1`]; `` — `"…"` is text,
`'…'` is a name, `` `…` `` is one character. Must be exactly one unit; with
`:=` Precog picks the unit. Owner, 2026-09-14.

### 2026-09-14 — `uenr`, Unbounded Exact Numerical Representation, from the standard Limb (owner)

An arbitrary-size number that never rounds, supplied by the std Limb rather
than the core language. Replaces the role of legacy's `bignum`/`bigint`.
Exact is exact: fractions are true rationals (owner, 2026-09-14).

### 2026-09-14 — A stored call wears the outer bracket: `var 'n' := [count[lend 'words']];` (owner) — superseded 2026-09-19: `let 'n' := count[lend 'words'];`

The `[ ]`-mandatory rule applies literally; a call's own bracket is its
argument list, not the value slot. Surfaced by the oracle (functions.md had
`:= add[10, 20]`); Claude offered both and leaned to the shorter; owner chose
the literal rule — one rule, no exception, and `[add[1] + 2]` never raises a
question.

### 2026-09-14 — Boolean literals are `true` / `false` (owner)

`var.bool 'correct?' = [true];`. Bare, reserved. Surfaced by the oracle —
no literal had ever been written down.

### 2026-09-14 — A number piece becomes text in its stored form, everywhere (owner)

`["x is " 'x']` holds `x is 0.1000000000000000055511151231257827021181583404541015625`
wherever it is built; `print` shows what a variable stores; `printu` alone
produces the useful form, for its own argument and its own pieces. Surfaced
by the oracle. Claude offered stored-by-default vs useful-when-building;
owner chose stored.

### 2026-09-14 — Shadowing only with `$:=` / `$=`; never for `immut` (owner)

First decided as "no shadowing", then refined the same day: a visible name
may be redeclared only with the `$`-marked operator, so hiding is always
deliberate and visible. An `immut` variable cannot be shadowed. Surfaced by
the oracle; owner's design.

### 2026-09-14 — `xx` chains are right-associative, as in mathematics; `( )` overrides (owner)

`[2 xx 3 xx 2]` = 512; `[(2 xx 3) xx 2]` = 64. Surfaced by the oracle.

### 2026-09-14 — A fresh value may be passed straight to a `lent` parameter (owner)

`count[["a", "b"]]` is allowed: the temporary lives for the call. No safety
issue — nothing else can reach it. Surfaced by the oracle; owner chose allow.

### 2026-09-14 — A lend may have its own name: `var 'r' := [lend 's'];` (owner) — now `let 'r' := lend 's';`

Surfaced by the oracle; owner chose allowed. Precog's lifetime inference
makes it safe in the compiler; the oracle refuses to return or store one.

### 2026-09-14 — A one-element list is `[1,]` (owner)

`[1]` is the number; `[1,]` is a list holding it. Surfaced by the oracle.
Also recorded: a `[ ]` inside a `[ ]` is a value (`[[1, 2], [3, 4]]`).

### 2026-09-14 — Mutability is statable (`var.mut`, `var.immut`) but not standard practice (owner) — now `let mut` / `let immut`

Plain `var 'x' := [1000];` is the norm; Precog infers. `immut` is a verified
contract (assignment is an error); `mut` states intent. Placed as a chain
segment before the type (`var.immut.int32`), following the existing chain
rule. No `const`: compile-time constants are Precog's to find. Owner's
decision on Claude's explanation of immutable-vs-constant.

### 2026-09-14 — Precedence: mathematics where settled, parentheses where not, parentheses override (owner)

`xx` > unary `-` > `x`/`/`/`mod` > `+`/`-` > comparisons > `not` > `and` >
`or`. Parentheses required for: `/` or `mod` beside another multiplicative
operator, `mod` beside anything, chained comparisons, `not` beside a
comparison. Claude offered a standard table vs none (recommending none);
owner chose the mathematical rule. Table in [design/operators.md](design/operators.md).

### 2026-09-14 — Builds are reproducible; requirements on Precog (owner)

Same source + compiler + named target → identical binary. Precog: no
wall-clock budgets (step counts only), pinned solver determinism (solver
version is part of the compiler version), compile-time I/O always a boundary,
target explicit and recorded, deterministic ordering, compile-time arithmetic
matching the target. JetPack lock file records a proof summary per site so
drift is visible. Raised via another session's question; Claude laid out the
requirements; owner adopted them. See
[design/reproducible-builds.md](design/reproducible-builds.md).

### 2026-09-18 — Structs (owner) — spelling revised 2026-09-19: `name: str.utf8`, `Locker[name = "A"]` (fields bare, later that day)

Blueprint `struct Name { type 'field' … }` — owner's shape: type first, no
`var` (a struct always owns its fields; Claude's reasoning, owner agreed),
no `;` (owner), `immut.` allowed on a field (owner: "allow it"). Values by
field name, every field required, each value in its own `[ ]` — owner chose
the no-exception form. Field access `'mine'.number` / `'mine'.number = [8];`
(owner confirmed the dot). Ownership: `give` on a field allowed and it
**consumes the whole variable** — owner's design, replacing Claude's
"forbid partial moves". See [design/structs.md](design/structs.md).

### 2026-09-18 — A struct is always owned (owner)

Even a struct of only plain fields needs the ownership verbs. Surfaced by
the oracle; Claude laid out plain-when-fields-are vs always-owned and
recommended always-owned (one rule; adding a field never changes call
sites); owner chose always-owned.

### 2026-09-18 — Compiler names: Cyborg Chassis Assembler (CCA), Nowork (owner)

The CyborgPL compiler is the **Cyborg Chassis Assembler**; N%'s compiler is
**Nowork**. Owner acknowledges "Assembler" may be confused with a real
assembler. Spelling "Chassis" vs the owner's "Chasis" to confirm.

### 2026-09-18 — A scripting language on top of CyborgPL (owner)

Exists; its writers never touch widths, streams, or `MAIN`; the final
product is compiled with Cyborg (full Precog); while iterating it compiles
itself, and that iteration compiler may assume what the real build will
prove. Resolves the fast-edit-loop concern without a Cyborg dev mode. Its own
iteration compiler **assumes the safest** (all checks on, nothing proven)
and hands off to Cyborg only for the final-product build (owner,
2026-09-18). Named **N%**: a separate language from both Cyborg and N#, with its own
designer-friendly syntax, fast compile, safest-by-default, performance
deferred to the Cyborg hand-off (owner, 2026-09-18). The hand-off is a
real translator. Open:
what exactly is hidden, boundary defaults, iteration/real-build agreement,
subset-or-own syntax, name. See [design/scripting.md](design/scripting.md).

### 2026-09-17 — Lists: two types, 1-based indexing, type-headed operations (owner) — element type bracketed since 2026-09-19: `list[int32]`

`list` (growable) and `array` (fixed), narrowing to the element type
(`var.list.int32`); with `:=` Precog picks (never-grown → `array`). Owner
chose two names over one after Claude laid out both and the everyday
meanings of the words. Indexing `'xs'[i]` is **1-based** — owner: "`'xs'[0]`
would be the value before the first, which doesn't exist"; Claude had
recommended 0 and withdrew it, since inclusive ranges make 1-based the
consistent choice. Operations are `list.length` / `list.append` /
`list.remove` / `array.length` — the type name as the chain's head (owner's
idea, "`list.` might just fit our lang"), `length` spelled out (owner: "`.length`
is better" than `len`). Language operations, not std. Element assignment,
empty list, boundary lengths remain open. See [design/lists.md](design/lists.md).

### 2026-09-17 — `1,000` is an error; leading zeros are illegal (owner)

A number followed by a comma and digits is an error pointing at `_`, rather
than the list `[1, 0]` the rules would otherwise make it; `000`/`007` are
not numbers. Surfaced by the OCaml oracle experiment.

### 2026-09-17 — Ownership verbs on a plain value are an error (owner)

`give 'n'` on an integer is a compile error, not a no-op: a plain value owns
nothing (no allocation behind it), so the verb would be false. Surfaced by
the OCaml oracle experiment, which allowed it. Owner chose error.

### 2026-09-17 — Comments: `#`, `#wl`, `#Nl`, `#Nld` (owner)

Owner's design, extending legacy's `#`/`#N`: `#` to end of line (trailing
allowed, unlike legacy), `#wl` whole line including preceding code, `#Nl` N
lines counting this one, `#Nld` this line plus N down. The oracle enforces
them.

### 2026-09-17 — Empty list: `= []` typed, `:= [list.empty]` untyped (owner)

Claude offered `= []` vs `= [list.empty]` for the typed form and recommended
the first; owner chose it and pointed out that under `:=` it must be
`[list.empty]`, since nothing on the left says "list". An empty `array` is an
error. See [design/lists.md](design/lists.md).

### 2026-09-17 — `pos` on every index; element assignment `'xs'[pos i] = […];` (owner)

Owner proposed `'xs'[pos 2]` and `'xs'[value-is 2]`. `pos` adopted everywhere
(reads and writes) — the bare `'xs'[2]` form is gone. `value-is` dropped:
Claude noted it hides a search with unanswered multiple-match questions;
owner: "can't you just use `if` for that one?" — a `perm` range loop finds
the position, then `pos` writes. No `list.find`. See [design/lists.md](design/lists.md).

### 2026-09-17 — Range loops; `temp`/`perm` mandatory on `for` and `range` (owner) — revised 2026-09-19: `for 'i' in range[a, b]`, `temp` is the default

`loop.temp.range 'i' [:= 0, := 1000] { }` — owner's shape. Decided one by
one: stop is **inclusive** ("what the writer sees": start from, stop at);
`temp`/`perm` **mandatory**; after `perm` the variable holds whatever the
loop last gave it (Claude's rule, owner confirmed); typed form
`loop.temp.range.int32 'i' [= 0, = 1000]` with the type last in the chain
(owner chose over `range`-last); `temp`/`perm` apply to `for` too; chain
order `loop.temp.seq.range.int32`; the body cannot assign to the counter
(Claude explained why; owner accepted); **step** exists as a third slot;
descending is a **negative step**; step 0 is an error. Parked: index
alongside element (needs indexing). Skipped: labelled `break`, character
ranges. `std::range[…]` as a value is superseded. The oracle enforces all of
it; see [design/control-flow.md](design/control-flow.md).

### 2026-09-16 — No development mode; Precog is optimised, never bypassed (owner)

Another session suggested a quicker path for everyday edits. Owner: "making
a shortcut for it is not Cyborg." Every build proves everything. Reusing
proofs for unchanged code is optimisation (same guarantees), not a
shortcut, and is allowed.

### 2026-09-14 — The oracle interpreter exists (owner)

`oracle/` — a small Rust tree-walking interpreter implementing exactly the
decided design, with `examples/*.cpls` (must run and match `.out`) and
`oracle/tests/reject/*.cpls` (must fail with the named message) as its test
suite. Owner asked for it so stale notes cannot drift from the language's
reality. It is the executable spec, not the compiler. Every pick it had to
make where the design is silent is listed in `oracle/README.md` and logged
below as open.

### 2026-09-14 — Function names are bare at the definition (owner)

`func.int32 add [...]`, matching the bare call `add[...]`. Quotes mean
variable; a function is not one. Function names are plain identifiers;
variables keep free naming. Owner asked whether functions should be quoted;
Claude laid out bare-both-ends vs quoted-both-ends (the latter needing a
separate index syntax); owner chose bare.

### 2026-09-14 — Automatic parallelism with an escape (owner)

Precog parallelises `loop.for` itself where ownership proves iterations
independent and the trip count makes it worthwhile; the runtime carries a
thread pool. The programmer writes nothing — except an opt-out chain segment
(`loop.for.seq`) for loops that must run in
order. Concurrency proper (async, channels, shared state) remains open.
Claude explained the concurrency/parallelism split and proposed this scope;
owner chose it. See [design/parallelism.md](design/parallelism.md).

### 2026-09-14 — Entry point simplified to `MAIN { }`; exit 0 implied; `std::exit[uint8]` otherwise (owner)

Supersedes `func.int32 MAIN` + mandatory `return` from earlier today. `MAIN`
is the program's block, not a function; ending it exits 0; other codes via
`std::exit[…]`, typed `uint8` (exit status is 8 bits on Unix — Claude's
correction of `int32`). Owner asked why a manual `return 0` exists at all;
answer: only because C made `main` a function. Uncallable by construction.

### 2026-09-14 — `lang:` removed; keywords are bare and reserved (owner)

The `lang:` prefix is dropped. Keywords — `func`, `var`, `return`, `if`,
`else`, `loop`, `in`, `break`, `continue`, `give`, `copy`, `lend`, `lent`,
`op`, `check`, `nocheck` — are bare and reserved, as in every mainstream
language. Claude laid out what `lang:` bought (zero reserved words, every
token self-describing) against its cost (five characters per statement, the
largest single source of verbosity, and the "esolang" first impression),
and offered keep / drop / shrink to `@`; owner chose drop: "I don't think
anyone would wanna name a function confusingly." Variables stay quoted;
`::`, `.`, `[ ]`, `{ }` unchanged. All samples in `design/` now read
without the prefix; earlier entries in this log that show `lang:` are
historical.

### 2026-09-14 — Control flow: `if` / `else.if` / `else`; `loop` / `loop.while` / `loop.for` (owner) — spelling revised 2026-09-19: `else if`, `while`, `for`

Branching as Claude proposed. Loops: Claude proposed three keywords
(`loop`, `while`, `for`); owner chose one, narrowed by the chain —
`loop`, `loop.while [cond]`, `loop.for 'x' in [list]`.
Conditions must be `bool`; braces mandatory; `break` / `continue`.
See [design/control-flow.md](design/control-flow.md).

### 2026-09-14 — Ownership transfer is explicit: `give` / `copy` / `lend` / `lend.mut` (owner) — parameter side revised 2026-09-19: `lent 'text': str.utf8`, owning is bare

A bare `'s' = ['t'];` on an owned (heap) value is a compile error; the
programmer writes `give` (move), `copy`, or `lend` /
`lend.mut` (borrow). Plain values copy with no keyword. Owner asked
whether a move needs a keyword; Claude recommended explicit and offered
Rust's words (`move`/`copy`/`borrow`) or owner's-perspective words
(`give`/`copy`/`lend`), recommending Rust's; owner chose owner's-perspective
as easier to understand. See [design/ownership-syntax.md](design/ownership-syntax.md).

Parameters declare what they accept (owner, 2026-09-14): `lent.str.utf8
'text'` receives a shared borrow, `lent.mut.…` a mutable one,
`var.…` ownership. Caller verb and parameter kind must agree. The
signature is the ownership contract.

### 2026-09-14 — No `++` / `+=` shorthand (owner)

Increment is `'i' = ['i' + 1];`. Claude laid out `++` (statement-only) and
compound assignment; owner considered `'i'++` and decided against any
shorthand.

### 2026-09-14 — Entry point is `func MAIN`, unquoted and uncallable (owner)

`func.int32 MAIN { … return [0]; }` — the return value is the exit
code, and **`MAIN` must declare `.int32` and return explicitly**; no implied
exit 0 (owner, after Claude offered both). `MAIN` is unquoted because it is the language's, not the
user's — a quoted `'main'` would be a reserved user name (owner's
observation). Owner proposed `START { }` or `func MAIN`; chose the
latter. `MAIN[]` is a compile error (Claude's recommendation; owner agreed):
one analysis root, one owner of top-level values, no legitimate re-entry.
A Limb with `MAIN` is a program; without, a library.

### 2026-09-14 — Local functions are called bare; other Limbs' via `::` (owner)

A function is a Limb member. Within its own Limb: `add[10, 20]`. From
elsewhere: `mathstuff::pow[2, 64]`. Quoted only at definition. Claude had
proposed `self::add`; owner chose bare, since `::` exists only for a real
separate `.limb`. Call vs index is told apart by shape, and so are calls and
operator words (a call is a word followed by `[`; an operator sits between
operands) — Claude first proposed reserving the six operator words as
function names; the owner pointed out the positions never collide, and the
exception was dropped. Zero reserved words, no exception.

### 2026-09-14 — `lang:` prefixes language built-ins (owner)

`lang:var.std::uenr 'n' = [1000];`, `lang:char.grapheme`, alongside
`std::print.stdout[...]`. Every path names its origin: `lang:` the language,
`Limb::` a Limb, `'…'` the user. Earlier `var …` samples read as `lang:var …`.
Scope: **every keyword** — `lang:func`, `lang:return`, `lang:if`,
`lang:check` (owner, 2026-09-14). Zero reserved words; a Limb may be named
`func`. All earlier samples read with `lang:` in front.

**Superseded the same day — see "`lang:` removed" below.**

### 2026-09-14 — `::` is the Limb boundary; `.` narrows within (owner)

Anything from a Limb is reached with `Limb::path`: `std::print.stdout[...]`,
`var.std::uenr 'n' = [1000];`. The chain continues across the boundary, so a
Limb type sits in a `var` chain like a built-in. Earlier samples written
`std.print.stdout` are superseded.

### 2026-09-14 — The typed form is always fully explicit (owner)

No bare `var.int`, `var.str`, or `var.char`: a typed declaration states the
width / encoding / unit (`var.int32`, `var.str.utf8`, `var.char.point`).
`:=` means Precog chooses; `=` means the programmer said everything. Claude
recommended; owner chose. Closes the width, encoding, and unit questions at
once.

### 2026-09-14 — `print` shows the real stored value; `printu` shows the useful one (owner)

`std::print.stdout[...]` prints exactly what is in memory — for a `bin` value
that is the full decimal expansion of the stored binary (`0.1` as `bin32`
prints `0.100000001490116119384765625`). `std::printu.stdout[...]` — `u` for
"useful" — prints the readable value (`0.1`). For `int`, `uint`, `str`, and
`deci` the two are identical; the difference only shows on `bin`. The honest
one gets the short name.

`print` on `bin128` dumps the full expansion — that is the point. `printu`'s
exact algorithm is Claude's call as long as the output is what a reader would
call useful; Claude will use shortest round-trip (Ryu-style). Owner, 2026-09-14.

### 2026-09-14 — Functions (owner) — spelling revised 2026-09-19: `fn add ['a': int32] -> int32`

`func.int32 'add' [var.int32 'a', var.int32 'b'] { return ['a' + 'b']; }`;
called as `add[10, 20]` (originally written `'add'[10, 20]`; superseded
the same day — see below). The chain narrows `func` by return type; omitted →
Precog infers. Parameters are `var` declarations; an untyped parameter's type
is inferred from all call sites. Inside `[ ]`, whitespace joins pieces into
one value and a comma separates values; several comma-separated values in a
bracket is a list. Digit grouping is `_`, not `,`. Claude proposed the shape
and `[a][b]` per-slot calls; owner chose comma-separated `[a, b]`. Full text
in [design/functions.md](design/functions.md).

### 2026-09-14 — Arithmetic operators (owner)

`+`, `-`, `x` (multiply), `xx` (power), `/`, `mod`, `==`, `and`, `or`,
`not`, `!==`, `<`, `>`, `<==`, `>==`. No `*`, no tetration. Word operators are
unambiguous because names are quoted. `<==`/`>==` are built from `==` because
a lone `=` is assignment and must never appear inside a comparison (owner
raised the inconsistency of `>=`; Claude proposed `>==`, words, unicode, or
none; owner chose `>==`). See
[design/operators.md](design/operators.md).

### 2026-09-14 — The compiler is verified by oracle + differential testing (owner)

For CyborgPL's own developers, not users: a reference interpreter (the
oracle, also the executable spec) and the optimised compiler are run on
generated programs and compared; any difference is a bug. Translation
validation of the LLVM half in CI; formal verification ruled out for now.
Claude proposed the layering; owner's framing was "oracle + optimised
generator". See [design/compiler-verification.md](design/compiler-verification.md).

### 2026-09-14 — Implementation languages (owner)

Rust for the compiler, `no_std` runtime, JetPack, test harness, and the
oracle interpreter (the oracle by a different AI model, no shared code); C++
only in a minimal LLVM wrapper; the standard Limb in CyborgPL. No C, no
OCaml. Claude first recommended Rust, then C++ throughout when the owner
raised "latest LLVM", then Rust + wrapper once the team shape was known;
owner chose Rust + wrapper. See
[design/implementation-languages.md](design/implementation-languages.md).

### 2026-09-14 — Review policy: human review follows risk (owner)

Tier 1, every line reviewed: Precog, borrow checker, proof-feeding into LLVM,
intrinsic contracts. Tier 2, tested and sampled: front end, runtime
arithmetic, JetPack, harness, oracle. Tier 3: the standard Limb is checked by
Precog itself. Claude proposed; owner chose. See
[design/review-policy.md](design/review-policy.md).

### 2026-09-14 — Licensing intent (owner)

Using the language: free, no credit anywhere. Modifying the compiler: free,
credit everywhere — source and every user-visible place. Claude mapped this
to Apache-2.0 WITH LLVM-exception (runtime, std Limb), Apache-2.0 plus an
attribution term (compiler), CC BY 4.0 (spec), trademarks (names), for
counsel to draft. See [design/licensing.md](design/licensing.md).

### 2026-09-14 — The compile-time technology is called Precog (owner)

The whole-program analysis/execution engine behind "everything at compile
time" is **Precog**. Claude proposed Precog, Foresight, Rehearsal, Ghost run,
Cortex and recommended Precog; owner chose Precog, accepting that the word is
shared with unrelated products.

### 2026-09-14 — Libraries are Limbs; the package manager is JetPack (owner)

A **Limb** is the unit of distribution, like Rust's crate — any library or program; `std` is one Limb. The package manager is **JetPack** (fast package),
manifest `JetPack.toml`. Source files are **`.cpls`** (CyborgPL Source; `.cpl`
avoided). A published Limb is a **`.limb`** file — the Limb directory packaged
for JetPack, always containing analysable source, never an opaque binary
(whole-program analysis must see inside it). See [design/tooling.md](design/tooling.md).

## Open questions

- SIMD leftovers: more element types (`simd[bool, N]` written directly?), shuffles and lane-crossing, masked load/store for a short tail, `simd` as a struct field or list element. (2026-09-22, `design/simd.md`)


- GPU offload (owner chose automatic, 2026-09-21): the offload test, the runtime, target descriptions for GPUs, what the body may contain. Not designed.

**Language — surfaced by the second (OCaml) oracle, 2026-09-17**

**Language — surfaced by the oracle**

**Language**
- N%: its syntax (undesigned), boundary defaults in the translation, translator verification. (owner's direction, 2026-09-18)
- Lifetime annotation spelling (optional, verified). (Claude, 2026-09-14)
- Generic `op`/`const`; std's standard interfaces; associated types. (2026-09-20)
- Concurrency: cancellation by hand; `whichever` with `send`; oracle support. (2026-09-22)
- Conversion, still open parts: a useful-form (short) number → text. (2026-09-19)
- A per-parameter `bare` flag (arguments written without quotes) — sketched for `std::c.printf`, advised against, shelved. (2026-09-19)
- Limb name declared in source (`limb [...]`)? Revisit with `pub` and the author's bare-reachability flag (`design/limbs.md`). (Claude, 2026-09-14)
- The Limb author's "reachable bare" flag — spelling, beside or on `pub`. (`design/limbs.md`, 2026-09-19)

**Precog**
- Non-terminating compile-time code: with step budgets now required, the budget's size and the error it produces. (Claude, 2026-09-14)
- Build output lists every `nocheck` site? Claude recommends yes. (Claude, 2026-09-14)

**Process**
- Build the oracle interpreter first, before the real compiler? Claude recommends yes. (Claude, 2026-09-14)
