# CyborgPL v2 — Design decisions

Running log. Each entry: date, the decision, who proposed it, and who chose it.
Nothing here is final until the owner says so; open questions live at the bottom.

## Decided

### 2026-09-30 — A lone target marked `default = false` means no default: a compile naming no target errors (owner)

Synthia claude found it: a lone target marked `default = no` (`false`) was
still the default by itself. Proposed (a) refuse the manifest when read;
(b) `false` means this Limb has no default, so only a compile that names
no target errors. Design claude recommended (a), since several targets
must have exactly one default. The owner: "`default = false` should error
when building?", "No, when compiling", and, asked which of `cca build`
with no target and with the target named errors: "no CLI target
specified with def false errors" — (b). Reading the manifest (`synthia
show`, the oracle's runs, which name no target) does not complain. The
several-targets rule (exactly one `default = true`) stands. A compile names its target with **`--target <name>`**, as in Cargo
and clang, once per command (compiler claude's spelling, left to it by
design claude; design claude recommended keeping it; the owner: "yea").
A named target must be one the Limb's `CCA.cplr` lists, or, for a plain
`.cpls` file, one in `cca/targets.txt` (compiler claude's reading).

### 2026-09-30 — `CCA.cplr` refuses a target CCA cannot build, as `Synthia.ttsr` does (owner: "refuse")

Compiler claude asked: CCA refused only a default target it could not
build, and accepted any other name (`x86_64-linux` with a runner) unused.
Design claude recommended matching `Synthia.ttsr`'s "Only what's actually
doable". The owner: "refuse". The one list is CCA's, `cca/targets.txt`,
which the oracle reads when it is built (`include_str!`) and Synthia
copies; today it holds `aarch64-macos` alone. Reject test
manifest_undoable_target; `examples/uses_a_limb` lost its `aarch64-linux`
target, which no longer passes.

### 2026-09-30 — Synthia is an automatic programmer: it does what a CCA user would, writing `CCA.cplr` too (owner: "yea")

Synthia claude, at the owner's asking, raised how CCA learns about a Limb
that has only a `Synthia.ttsr`, since CCA never reads one. Proposed: (a)
Synthia writes a `CCA.cplr` for each Limb that has none; (b) `cca build`
takes Limbs on its command line (`--limb cash2=.synthia/money`); (c) a
Limb must carry a `CCA.cplr` to be used. The owner: "`CCA.cplr` is
already hella built-in. So, you could just go the programmer way ...
Synthia just becomes like an automatic programmer". Design claude
restated it as (a), as a general rule — Synthia does whatever a
programmer using CCA would do by hand, writing a `CCA.cplr` where one is
needed, and CCA never knows Synthia exists (as the override, 2026-09-29) —
and the owner: "yea". **The project's own `CCA.cplr` is written beside `Synthia.ttsr`**
(owner: "`a`"; proposed (a) there, rewritten from `Synthia.ttsr` each
time with a first line saying it is generated, design claude's pick; (b)
in `.synthia/`, with a new `cca build --manifest …` flag; (c) a copy of
the whole project in `.synthia/`, which would point messages at the
copy). Edits go in `Synthia.ttsr`. Its first line (relayed by synthia claude, which
proposed it; the owner: "yea"):
`# Written by Synthia from Synthia.ttsr, and rewritten each time: edit Synthia.ttsr, not this file.`
A `CCA.cplr` beside `Synthia.ttsr` that lacks that line (not Synthia's)
is left alone: Synthia refuses to write, saying to delete or move it, and
a command overrides that (relayed by synthia claude; proposed (a) refuse,
(b) overwrite, (c) back it up to `.synthia/` and overwrite; synthia claude
recommended a; the owner: "`a`, and there should be an override command").
The command is **`synthia takeover`** (relayed by synthia claude;
proposed (a) `takeover`, (b) `overwrite`, (c) `claim`; synthia claude
recommended a, a one-time handover; the owner: "`a`"): it writes
`CCA.cplr` from `Synthia.ttsr`, replacing any file there, and the header
marks it Synthia's from then on. A `CCA.cplr` is Synthia's when its first
line begins `# Written by Synthia`. Its success message is synthia
claude's provisional wording.

### 2026-09-30 — A `Synthia.ttsr` target's `default` is `yes` or `no` (owner: b)

Relayed by synthia claude. Proposed: (a) only `yes`, the line left out
for "not the default"; (b) `yes` or `no`, as `CCA.cplr`'s `true` or
`false`. Synthia claude recommended (a). The owner: "`b`". Open, the same
in both manifests: a lone target marked `default = no` (`false`) is still
the default by itself, as the oracle counts defaults only among several;
whether that contradiction is refused has not been asked.

### 2026-09-30 — A renamed copy lives in the project, in Synthia's own `.synthia/` folder (owner: a)

Relayed by synthia claude. Where the copy made by `money.prefix = cash2`
lives: proposed (a) inside the project, in a folder of Synthia's own next
to `Synthia.ttsr`, shown as `.synthia/`; (b) a shared folder in the home
directory, `~/.synthia/`; (c) next to the original, `../money-cash2`.
Synthia claude recommended (a): the rename is the project's choice, and
all Synthia makes stays in one deletable folder. The owner: "`a`".
Deleting it is always safe (Synthia remakes the copy), and it is kept out
of git.

### 2026-09-30 — Synthia's prefix override is a dotted `prefix` line on the needed Limb's key (owner: "Prefix shall be narrowed using a dot", "yea")

Relayed by synthia claude. Proposed: (a) the word before the dot is
always what code writes (`cash2.path = ../money`); (b) that word is this
manifest's label for the needed Limb, and a `prefix` line overrides.
Synthia claude recommended (a). The owner: "Prefix shall be narrowed using
a dot"; asked whether that means (b): "yea".

```
[my needed things]
money.path = ../money
money.prefix = cash2
```

Code writes `cash2::`; Synthia copies `money`, sets `prefix = cash2` in the
copy and hands CCA the copy (entry of 2026-09-29, 6bdcb33). Without the
line, code writes `money`'s own prefix. The label (`money`) keeps the name
rule. Open: where the renamed copy lives (synthia claude's, next).

### 2026-09-30 — `cca prepare` names Limbs by their `name`; a shared name is refused, listing the prefixes (owner: a)

After the prefix replaced the local name, `cca prepare`'s names had become
prefixes (design claude's reading, 2026-09-29). The owner: "Shouldn't it
be name, not prefix?". Design claude recommended the prefix, being
unique in a build and one word; the owner: "I think name's better,
because why would there even be a name? Only for search?", then made names
strict (entry below). For two dependencies sharing a name, proposed (a)
refuse and list the matching Limbs by prefix, (b) run all of them, (c) make
a shared name an error for the whole build. Design claude recommended a;
the owner: "`a`". Then one is picked by a flag, `cca prepare --prefix
geo2` (proposed (a) the bare prefix, which a name can collide with, (b)
the flag, design claude's pick, (c) the Limb's folder, not there for
registry Limbs; the owner answered "`v`", asked b or c: "flag"). Then
compiler claude's question: `cca prepare money` with a folder `money/`
beside it. Proposed (a) a folder always has a `/` (`./money`, `../app`),
(b) a flag, (c) CCA guesses from the folders there. The owner offered
`--folder`; asked how `cca run` takes a folder (a plain path, its first
word, since only a program can stand there), design claude set out
`--folder` on `cca prepare` only against a `/` in both commands, leaning
to the `/`. The owner: "`/`". **A word with a `/` is a folder; a bare word
is a Limb's name**: `cca prepare ../app geometry`. (Design claude's
record first also gave `cca prepare ./money`, a folder with no names; that
is the still-open question of `cca prepare` with no names, not part of
this ruling.) `--prefix` takes a comma list, as names do: `cca prepare
--prefix geo2, geo3` (compiler claude asked; design claude recommended
it; the owner, 2026-09-30: "yea"). Names and `--prefix` mix in one command: `cca prepare money
--prefix geo2` (compiler claude asked; design claude recommended it; the
owner: "yea"). The folder comes first, as the program does in `cca run`:
`cca prepare geometry ../app` is refused, "the folder comes first"
(compiler claude's provisional pick; design claude recommended keeping
it; the owner: "yea"). A bare `.` or `..` is a folder too, the one exception to the
`/` rule, since no Limb can be named that (compiler claude had refused
them with a hint to write `./`; design claude recommended accepting them;
the owner: "yea"). Names come before `--prefix`, and every word after it is a
prefix, once per command: `cca prepare --prefix geo2 money` is refused
(compiler claude's provisional pick; design claude recommended keeping
it; the owner: "yea").

### 2026-09-30 — A Limb's `name` stays strict, in both manifests (owner: "strict")

The module-name rule for `name` (2026-09-29, entry below) had as its reason
that code writes the name before `::`; the prefix took that job, and the
oracle accepted any text in `CCA.cplr`'s 'name' while Synthia refused it.
Asked, while weighing whether `cca prepare` takes names or prefixes, "Is
the limb name rules strict?", design claude laid out strict (one plain
word, no quoting on a command line; two dependencies may still share one)
against free ("Areas of shapes", easier to search, awkward to type). The
owner: "strict". So in `Synthia.ttsr` and `CCA.cplr` alike a Limb's `name`
is lowercase ASCII letters, digits and `_`, starting with a letter, not a
keyword, not a Windows device name. The oracle refuses one otherwise
(reject test manifest_bad_name); the example Limb is `areas_of_shapes`,
prefix `areas`.

### 2026-09-29 — In `Synthia.ttsr` the word before `::` is `prefix = cash`, in `[my identity]` (owner: "`prefix`")

Relayed by synthia claude. Proposed: (a) `prefix`; (b) a phrase in the
Limb's own voice, such as `called_in_code`. Synthia claude recommended
(a), one word across both managers. The owner: "`prefix`". Synthia
claude's own step, which the owner may overrule: `prefix` follows the
module-name rule, as `CCA.cplr`'s does in the oracle. **`prefix` is required in every `Synthia.ttsr`**, as in
`CCA.cplr` (owner: "Well, if it's not always required, it could be weird
to CyborgPL users"; synthia claude had first offered "required only when
another Limb needs it", then withdrew it).

### 2026-09-29 — A `Synthia.ttsr` target must be one CCA can build (owner: "Only what's actually doable")

Relayed by synthia claude. Proposed: (a) only targets CCA knows, from a
list; (b) the shape `architecture-system`; (c) anything, CCA judging at
build time. Synthia claude recommended (b) until CCA had a list. The
owner: "Only what's actually doable". Synthia claude has asked compiler
claude for CCA's list, its spellings and where it lives. Open: whether
`CCA.cplr` refuses a target CCA cannot build (not ruled for CCA; the
oracle, which builds nothing, cannot tell today).

### 2026-09-29 — The word in front of `::` is the Limb maker's, and apart from its name (owner: "yea")

Asked whether a dependency found by folder may call itself something other
than the word a user's manifest lists it under (the oracle refused that,
design claude's pick from before names could repeat), the owner asked
where the word in front of `::` is decided in a writer's code (nowhere:
source never names Limbs; the manifest does), weighed the downsides, then
asked whether the Limb maker could write it: "Infront and name are
different things". Design claude restated: a Limb's manifest holds its
`name`, a label for people that may repeat, and, separately, the word
code writes in front of `::`, which its maker picks. The owner: "yea".
In `CCA.cplr` it is **`const 'prefix' := "cash";`** (owner: b; proposed
(a) `'code_name'`, design claude's pick, (b) `'prefix'`, (c) `'infront'`;
dots are not allowed in a quoted name). The owner said to pass it to
synthia claude, whose question the `Synthia.ttsr` spelling is. **Every
`CCA.cplr` has a `'prefix'`** (owner: "required"; proposed (a) required,
(b) optional with a fallback, which has no good source since `name` is a
label; design claude recommended a).
**No override: two Limbs with the same prefix are an error, and the fix
is to edit one Limb's source** (owner: "Just edit the source code";
proposed (a) a `prefix` in the dependency line only to override, design
claude's pick, (b) no override, (c) every dependency line states its
prefix). The owner pointed out a Limb's files can simply be edited; a
registry Limb is copied to a folder first, since the downloaded one is
checked against the registry's checksum. So a dependency line names no
word: `cca::Dep[path = "../money"]` or `cca::Dep[id = "…", version = "…"]`. **A Limb may pick `std` or `cca`** (owner: "They can, it just wouldn't
work practically"; design claude had recommended refusing them): using it
clashes with the built-in one, like any two Limbs with one prefix. In the
oracle: `'prefix'` is required and follows the module-name rule; a
`cca::Dep` naming a word is refused; a dependency without a `CCA.cplr` is
refused (no prefix to reach it by); reject tests manifest_no_prefix,
manifest_bad_prefix, limb_dep_names_word, limb_prefix_clash,
limb_prefix_std. The check that a folder Limb's own name matches its local
name is gone with the local name. Possibly reopening: asked by synthia claude whether
`Synthia.ttsr` follows the no-override rule, the owner said "this is
Synthia. So... You could do an easy override"; offered an override in
both manifests (reopening this) or in neither, the owner: "Undecided yet,
wait". Then, asked by design claude how Synthia could offer
an easy override: proposed (1) an override in both manifests, reopening
this ruling, or (2) Synthia doing the edit for the user — copying that
Limb into a folder of its own, changing `'prefix'` in the copy, and
handing CCA an ordinary path dependency (design claude's pick). The
owner: "I want the one that Synthia just does what the CyborgPL
programmer had to do, when they were using CCA's PM" — (2). **CCA.cplr
keeps no override; Synthia's override is the copy-and-edit a CCA user
does by hand.** Open: its spelling in `Synthia.ttsr` (synthia claude's).
Open: the `Synthia.ttsr` spelling (synthia claude's); whether `name`, no
longer written in code, still follows the module-name rule (entry below).

### 2026-09-29 — A Limb's own `version` is any text, at most 256 bytes (owner: "256 bytes it is")

Relayed by synthia claude. Proposed: (a) three numbers, `1.0.0`; (b) one
to three numbers; (c) anything. The owner first said "Maximum is 4 dots,
numbers only", then, before anything was built: "let's just do anything
with a character byte limit", "it doesn't require `.`". Weighed: 8, 16,
24, 32, 64 bytes. 8 refuses `12.10.100` (9 bytes) and `1.0.0-rc.1` (10);
`1.0.0-beta.12+build.2026.09.29` is 30; the shortest Thai version,
`เวอร์ชัน ๑.๒.๓`, is 36, so 24 and 32 refuse any version written in
Thai (byte counts checked by design claude). Synthia claude recommended
256 if Thai matters. The owner: "256 bytes it is". This is the Limb's own
`version` only; what `geometry.version = 1.2` means for a needed Limb is
left for the registry. `CCA.cplr`'s 'version' had no limit in the
oracle; asked whether CCA gets the same limit, the owner: "same". The
oracle refuses one over 256 bytes (reject test manifest_long_version).

### 2026-09-29 — `CCA.cplr` needs `name`, `version` and a target too (owner: "yea")

Synthia claude noted that the oracle required none of these in `CCA.cplr`
while `Synthia.ttsr` now does (entry below), so CCA accepted a manifest
Synthia refuses. Design claude recommended matching, so a Limb that
builds under one package manager builds under the other. The owner:
"yea". The oracle refuses a `CCA.cplr` without `name`, `version`, or a
target (reject tests manifest_no_name, manifest_no_version,
manifest_no_target); a folder with no `CCA.cplr` is still a plain program.
`tooling.md`.

### 2026-09-29 — A Limb's name and a needed Limb's local name follow the module-name rule (owner: "yea")

Relayed by synthia claude. In `Synthia.ttsr`, `name` and the key before
`.id`, `.version` or `.path` in `[my needed things]` follow CyborgPL's rule
for module names: lowercase ASCII letters, digits and `_`, starting with a
letter, not a keyword, not a Windows device name. Synthia claude's reason:
a local name is written in code as `geometry::`, and a Limb's own name
becomes the local name of every Limb that needs it. The owner: "yea".
Synthia gives the oracle's reasons word for word, and its keyword list is
copied from `oracle/src/parser.rs` at 898ad6d: design claude tells synthia
claude when a keyword is added.

Found alongside, open: the oracle refuses a `path` dependency whose own
`CCA.cplr` name differs from its local name ("the Limb at `../x` calls
itself `a`, not `b`"), a check design claude added on 2026-09-29 (a `cca::Dep`
entry above) before the owner ruled that names may repeat and the local
name is each Limb's own label. The two disagree; which is right is the
owner's.

### 2026-09-29 — A `Synthia.ttsr` needs `name`, `version` and a target; `id` and times only to publish (owner: b)

Relayed by synthia claude. Proposed: (a) everything in the design's
example; (b) what a build needs, with the ID and times checked when
publishing; (c) nothing, as CCA reads `CCA.cplr` today. Synthia claude
recommended (b): a Limb that stays on one machine (`money` at `../money`)
should not need a random 64-digit ID and times. The owner: "`b`". A
missing one is refused with a `why:` line, CCA's label (synthia claude's,
as built):

```
error: `[my identity]` has no `version`
where: Synthia.ttsr, line 1, column 1 — in [my identity]
code:  [my identity]

why:   a build needs the Limb's `name` and `version`; only publishing needs its `id` and times
```

Noted by synthia claude: the oracle required none of `name`, `version`,
`targets` in `CCA.cplr`. Settled the same day: it matches (entry above).

### 2026-09-29 — `synthia show` prints the manifest exactly as written (owner: "Raw")

Relayed by synthia claude. Proposed: (a) the file itself; (b) a
plain-words summary ("needs: geometry 1.2, from the registry", the
targets and the default). Synthia claude recommended (b). The owner:
"Raw"; asked whether that means exactly as written rather than tidied:
"exactly, because there's `--readable` for the more readable ver". So
`synthia show` prints the checked manifest with its comments, blank lines
and order kept, and `--readable` changes only the time values
(`time.created = 29 September 2026, 17:40 (UTC+7), 2 hours ago`). As
synthia claude built it (feature.cli.show), not ruled: every other byte
stays, line endings included; a wrong manifest prints its problems in
CCA's shape and exits 1; it reads `Synthia.ttsr` in the current folder.
Its wording for no command, an unknown command or flag, and no manifest
("error: there is no Synthia.ttsr in this folder") is provisional,
not yet asked.

### 2026-09-29 — Readable times come from `synthia show --readable` (owner)

Relayed by synthia claude. Replaces the "another command" part of the
entry below; the readable form itself stands. Synthia claude proposed
`synthia time`; the owner floated "readable"; synthia claude set out
`synthia readable` beside `synthia time`; then the owner proposed
`synthia show --readable`, and, asked whether that replaces "another
command": "Yeah, not a separate command, just not exactly the same one".
So `synthia show` gives times as written (`2026-09-29T17:40:00+07:00`),
and `synthia show --readable` gives `29 September 2026, 17:40 (UTC+7),
2 hours ago`. Open: the layout of `synthia show`.

### 2026-09-29 — A manifest time reads in its own zone, plus how long ago, from a command of its own (owner: "`c`, but it should be another command")

Relayed by synthia claude. Proposed: (a) the zone written in the file
only; (b) converted to the zone of the computer running it; (c) (a) plus
how long ago. Synthia claude recommended (c). The owner: "`c`, but it
should be another command". So `2026-09-29T17:40:00+07:00` reads
`29 September 2026, 17:40 (UTC+7), 2 hours ago`, shown by a command other
than `synthia show`. The clock is read only for "ago", on display; builds
never read these times. Open: that command's name, and what `synthia
show` does with times (as written, or left out).

### 2026-09-29 — A `#` inside a `Synthia.ttsr` value is part of it (owner: "yea")

Synthia claude's fix for a comment after a value (acb3aa9): a `#` that
starts a value, or follows a space or tab, is a comment in the wrong
place and refused; a `#` inside a word stays in the value, so
`money.path = ../money#2` is the folder `../money#2`. Design claude
brought it to the owner as a rule users meet, recommending it, since a
path or text can hold a `#`. The owner: "yea".

### 2026-09-29 — `synthia show` shows a manifest, times readably (owner: "opt 1")

Relayed by synthia claude. Proposed: `synthia show`, `synthia info`,
`synthia manifest`; synthia claude recommended `show`, the plainest word,
and one that could later show a needed Limb (`synthia show geometry`).
The owner: "opt 1", which synthia claude reports as `show`. The owner
added: "there should be a way to view the recorded time in an
easier-to-read format", which synthia claude took as `synthia show`
giving the times readably. Superseded the same day: the readable time is a
command of its own (next entry up).

### 2026-09-29 — A refused `Synthia.ttsr` is reported in CCA's message shape (owner: "yea")

Relayed by synthia claude. Proposed: a one-line form
(`Synthia.ttsr:3:5: the `=` has one space on each side`), or the labelled
shape of `design/diagnostics.md`. Synthia claude recommended CCA's shape
— one shape for both tools, and the owner had found one-liners "less
useful than C's" (2026-09-22). The owner: "yea".

```
error: the `=` has one space on each side
where: Synthia.ttsr, line 3, column 5, in [my identity]
code:  name=shop

fix:   line 3:
       name = shop
```

`fix:` is left out when there is no one right line to write.

### 2026-09-29 — `Synthia.ttsr` lines are written one way; anything else is refused (owner: "refuse")

Relayed by synthia claude. `synthia-manifest.md` shows `key = value` with
one space each side of `=`, no indentation, and comments on their own
line. Synthia claude asked whether `name=shop`, an indented line, or a
comment after a value (`geometry.version = 1.2  # pinned`) should be
refused or accepted, and recommended refusing: loosening later breaks no
manifest, tightening would. The owner: "refuse". Synthia's refusal shows
the corrected line where there is one (`name=shop` → `name = shop`):
synthia claude's, and the owner, asked afterwards, did not recall ruling
it but agreed: "I think it's correct".

### 2026-09-29 — Manifests stay data; a Turing-complete `build.cplr` runs only when asked (owner)

The owner felt the manifest "should be turing complete, like the built-in
PM". Design claude pointed out `CCA.cplr` is not: it holds plain `const`
settings only. Computable manifests (Gradle, `setup.py`, `build.zig`, Nix)
run someone's code on install, cannot be read without running, and fight
reproducible builds; Cargo, `package.json` and `go.mod` stay data. The
owner: "Make it only run the not turing complete one? The turing could be
separate." So **both manifests stay data**, read on fetch and install, and
**a build program lives apart**. When it runs: proposed (a) only for your
own Limb, (b) for dependencies too, as Cargo's `build.rs`, (c) dependencies
only if allowed; the owner: "It's a special command to run, either run
specific ones, or run everything" — **it never runs by itself**. Its file:
proposed `build.cpls`, `setup.cpls`, or tied to the manifest's name; the
owner asked whether `.cplr` must change (no: `.cplr` is CyborgPL-syntax
tooling, the name saying which), and design claude proposed
**`build.cplr`**, a full CyborgPL program beside the data-only `CCA.cplr`;
owner: "yea". Both package managers run it (parity). The command is
**`prepare`** (owner: b): `cca prepare geometry money`, `cca prepare --all`,
`synthia prepare` alike; proposed alternatives `build-script`, `run-build`.
Then compiler claude's four suggestions, which the owner accepted ("Yeah,
but number 2 needs `,`"): `prepare` acts on the current directory's Limb
or a folder given; names are dependencies' local names (or the Limb's
own), **separated by commas** (`cca prepare geometry, money`); a
`build.cplr` runs in its own Limb's folder; `--all` goes dependencies
first, each once, stopping at the first failure. `tooling.md`.

### 2026-09-29 — One target is marked the default (owner: b)

Asked by design claude and, the same moment, by compiler claude: with
several targets and none named, which is built? Proposed (a) this
machine's, if listed (compiler claude's suggestion; it takes the target
from the host, which reproducible builds forbid); (b) one marked
default; (c) the first listed; (d) none. Design claude recommended b;
owner: b. `aarch64-macos.default = yes` in `Synthia.ttsr`, `default =
true` on one `cca::Target` in `CCA.cplr` (a CyborgPL bool, design
claude's spelling there). With several targets exactly one is marked; the
oracle refuses otherwise (reject test manifest_no_default).
`synthia-manifest.md`.

### 2026-09-29 — `Synthia.ttsr` comments are `#`; names may repeat, IDs never (owner)

Comments are `#` with a space after it, as in CyborgPL (owner; proposed
alternatives `//`, none). A Limb's name: proposed (a) unique in the registry
too, first come, as crates.io; (b) a label only, the ID being the identity,
with each manifest's key before `.id` naming the dependency for its own
code. Design claude recommended b; owner: "Names could be the same, IDs
can't." So a registry dependency in `CCA.cplr` carries its ID too,
`cca::Dep[name = "geometry", id = "…", version = "1.2"]`, and the oracle
refuses one without a 64-hex-digit ID (reject test limb_registry_needs_id).
`synthia-manifest.md`.

### 2026-09-29 — Several targets; CCA's package manager can do whatever Synthia can (owner: b, "yea")

`[my target]` in `Synthia.ttsr` holds several targets, each with its
settings as dotted keys (`aarch64-linux.runner = qemu-aarch64`) (owner:
b; alternative: one target). The owner then asked whether CCA's package
manager can do "some Synthia shit": not yet — `CCA.cplr` had one `target`,
which CCA refused as "not yet" — and "if not, that feels like the base lang
not being able to do inline ASM". Design claude proposed the rule that
CCA's package manager can do everything Synthia can, in its plainer
CyborgPL-syntax way, Synthia's edge being convenience, never capability
(which also keeps "CCA never depends on Synthia"); owner: "yea". So
`CCA.cplr`'s `target` became `targets`, a list of `cca::Target[name = …,
runner = …]`; the oracle checks their shape and refuses the old `target`
(reject test manifest_one_target). `synthia-manifest.md`, `tooling.md`.

### 2026-09-29 — A Limb states its own ID; a registered ID is refused twice and kept for good (owner)

`id = …` sits in `[my identity]` (owner: a; alternative: only in the
registry), generated once and never changed. The owner asked whether
matching IDs are allowed and whether a match is basically impossible: by
accident, yes — design claude first answered with uncomputed figures
(2²⁵⁶ "roughly the atoms in the universe", a 10⁻⁵⁰ chance); the owner:
"You better compute your answers". Computed: 2²⁵⁶ ≈ 1.16 × 10⁷⁷, about a
thousandth of the ~10⁸⁰ atoms, and 8 × 10¹⁸ Limbs collide with chance
≈ 2.8 × 10⁻⁴⁰. The real risk is a copied ID, so **the registry refuses an ID
it already has**, and **a registered ID is permanent** (owner: "yea, and a
registered ID is perm"). `design/synthia-manifest.md`.

### 2026-09-29 — `Synthia.ttsr`'s syntax, begun (owner)

The owner started from their own sketch — `[my credentials]` with `name`,
`version`, `time.created`, `time.published`, `time.stated_ver.created`,
then `[my needed things]` and `[my target]` — over design claude's three
shapes (TOML-like, indented, braced blocks; it recommended TOML-like
without quotes). Settled, one question at a time:
`[my identity]` (b, over keeping "credentials", which reads as secrets);
`time.stated_ver.created` is when the stated version was made; the
publish time is the registry's (b); times are ISO 8601 (a), a readable
form being a Synthia command's job (the owner first chose a readable
second line, then withdrew it); a dependency is dotted keys (b); a
published dependency is named by its **ID, which the Limb's creator
generates: 256 bits of randomness as 64 hex digits, required to register**
(the owner); the ID takes no quotes or braces (the owner, after design
claude argued for `{}` — "you said the ID would look like any other text
value; it won't, there's `.id`"). `design/synthia-manifest.md`.

### 2026-09-29 — Command-line arguments: `MAIN ['argc': int32, 'argv': ptr[ptr[uint8]]]`, C's own (owner: "low-level shit, decided")

`cca run` could not pass arguments: the language had no door for them. The
owner asked what they are (the words after a program's name), whether
reading them is not written in CyborgPL (the understanding is — flags,
subcommands, a friendly list, in std or a Limb; the raw words reach only
the runtime at start-up, so the language needs one door), and why not
"the low-level shit". Proposed: (a) `std::args[]` (a std question, so
out); (b) `MAIN ['args': list[str.utf8]]`, a ready list (design claude's
recommendation); (c) `MAIN ['argc': int32, 'argv': ptr[ptr[uint8]]]`, C's
own data; (d) nothing until std. The owner: "isn't it the base lang, it's
even lower than std. It's not like we're in the Lua level, we're at Rust,
C, C++ level", then "low-level shit, decided". So MAIN may take the two
parameters, exactly these types (names free); as in C, the program's name
comes first, each word ends in a NUL, and `argv[argc]` is a null pointer
(design claude's reading of "C's own"). Reading them is `nocheck`. A MAIN
without parameters stays as it was. `examples/args.cpls` (its `.args`
holds the words the test passes; `oracle file -- words`); reject tests
main_args_type, main_args_one. `functions.md`.

### 2026-09-29 — A dependency may have a MAIN, which runs only when that Limb itself is run (owner: a)

CCA refused a MAIN in a Limb others depend on; the oracle dropped it;
neither was ruled. Proposed (a) allowed and ignored when used as a
dependency, so a library can ship its own demo or tool (Cargo's library
plus binary); (b) refused, a dependency is a pure library. Design claude
recommended a; owner: a. The oracle already does this. `limbs.md`.

### 2026-09-29 — Tests: `test fn`, and the `must` family (owner: b, a, "the fam")

Compiler claude asked what `cca test` runs. Proposed (a) programs with
expected output, (b) test functions with assertions, (c) both; design
claude recommended c; the owner asked "assert_eq?" and chose b. A test is
`test fn name [] { … }` (owner: a; alternatives: a `module tests` block, a
`#test` file mode). The assertion: `expect` ("sounds like an optimization
hint", owner), `assert`, `must`, `ensure` were proposed; the owner asked
about `must_be` / `must-be` and then `must.`, and chose the family: `must
<condition>;`, `must.fail …;`, `must.fail.<reason> …;`, `must.stop …;`.
Only inside a `test fn`, never `pub`, never called, left out of builds
(design claude's details). `design/testing.md`; the oracle runs them with
`--test`; `examples/testing.cpls`; four reject tests.

### 2026-09-29 — The Synthia logo: four even orbits round a star, on a deep-space tile (owner)

Made by the owner with website claude (`brand/synthia/`, 7870adb): the
owner's pick "Star at the heart · 4 loops" — four even orbits 45° apart,
each a shade along cyan → violet → pink with one glowing atom riding it,
round a four-point star, on a deep-space tile (Lunae's cosmic palette,
CyborgPL's tile shape). Tried before it: a dolphin, a sun/moon emblem,
sound waves (dropped as too generic after a reverse image search),
S-shaped waves, a constellation, a molecule. The owner asked for
something closer to React's logo; website claude flagged that React's
mark (three equal ellipses and a centre dot) is Meta's trademark, and four
loops and a star keep it apart. The owner chose it knowing the risk. At the
owner's word, relayed by website claude ("say that it should be put in the
Synthia GH repo"), it is in the Synthia repo's `brand/` and at the top of
its README (Synthia 98a2549).

### 2026-09-29 — A `cca::Dep` is a name plus a `version` or a `path`; the oracle runs Limbs with dependencies (owner: a)

Asked whether Limbs were ready for compiler claude: the language rules
were (`limbs.md`), but not what a dependency in `CCA.cplr` holds, and the
oracle ran one Limb only. Proposed (a) a name plus either `version` (from
the registry) or `path` (a folder on this machine), Cargo's model, so two
Limbs can be developed together and CCA can learn dependencies before any
registry exists; (b) registry only. Design claude recommended a; owner: a
(after an `s`, which design claude asked about).

```
const 'dependencies' := [
    cca::Dep[name = "geometry", version = "1.2"],
    cca::Dep[name = "money", path = "../money"],
];
```

The oracle now reads `CCA.cplr` and loads each `path` dependency (and
theirs) beside the Limb being built: `areas::circle[…]`,
`areas::Circle[radius = 2.0]`, `'c': areas::Circle`, `areas::'pi'`,
`areas::shapes.square[…]`; only `pub` crosses a Limb's edge, and `limb.`
inside a dependency is that dependency's root. It refuses a `version`
dependency (it cannot fetch), an unknown setting, `pub const`, a
dependency naming both a `path` and a `version`, a cycle, and a name that
differs from the dependency's own `CCA.cplr`. `examples/uses_a_limb/` with
`examples/limbs/areas/`; seven reject tests (limb_*, manifest_*).

### 2026-09-29 — The Limb registry is an index that links to each Limb's own release (owner)

The owner asked how to host a Limb registry without paying, apart from
the domain (`limbs.xxx`, bought when a registry exists). Proposed: (1) a
registry that is an index only, small files naming each Limb's versions,
where each release lives and a checksum, served as static files
(Cloudflare Pages, free); the code stays with its author; (2) a registry
with uploads on Cloudflare's free tier (Workers, R2, D1). Asked what is
better past 10 GB (the index-only design never gets there; R2 or
Backblaze B2 for a few dollars; sponsorship programs such as Fastly's for
large public registries), the owner chose "the 'cheating' route",
confirmed as "the one that just links to someone's Limb release". The
registry never stores code. Then, the same day (owner: "1. GH 2. yea
3. registering is a PR, updating needs no review again. And please do
note that Malware-free is not [guaranteed]"):
- **A release lives on GitHub**, and nowhere else. Proposed alternatives:
  also GitLab, Codeberg, any link.
- **The index keeps a checksum of every release**, so a file swapped
  behind its link is caught. (Design claude recommended it strongly: a
  linking registry is only as trustworthy as its checksums.)
- **Registering a new Limb is a pull request**, reviewed; **a new version
  of a registered Limb needs no review**. How an update proves it comes
  from the Limb's owner is open.
- **Malware-free is not guaranteed**: the registry says so plainly.
`tooling.md`.

### 2026-09-29 — Synthia keeps CyborgPL's licence; its repo is set up (owner)

Setting up `Cyborg-Assembly-Manufacturing-Site/Synthia`, the owner asked
whether the "average package manager licence" is MIT (the most common:
pnpm, pip, Composer, Bundler; Cargo is MIT or Apache-2.0, Yarn and
Homebrew BSD, npm Artistic-2.0), what MIT does, and what Apache's patent
grant means (a contributor cannot later sue users over patents in their
own contribution). Owner: "Oh hell no, same license, then": Apache-2.0
WITH LLVM-exception, as everywhere (2026-09-23). The owner also set the
repo's description: "Fast, reliable, and easy-to-use dependency
management for CyborgPL." The repo holds a README in CyborgPL's voice,
the licence, a Go module and the session's `CLAUDE.md`; its brief is
`design/synthia-session.md`. The session is not started.

### 2026-09-29 — CCA runs and tests too; it never depends on Synthia (owner)

Compiler claude's brief said "CCA never runs what it builds: running,
testing, `--input` and the per-target `runner` are Synthia's"
(2026-09-22). With CCA's own package manager, design claude asked whether
CCA should also run and test, so a small project never needs Synthia.
Owner: "Yeah, CCA doesn't need to depend on Synthia." So `cca run` and
`cca test` exist beside `cca build`; how `--input` and a cross target's
`runner` work in CCA is open. `compiler-session.md`.

### 2026-09-29 — CCA's package manager is compiler claude's; Synthia gets its own session (owner)

Compiler claude's brief had said "CCA and Synthia". With two package
managers and Synthia in Go, design claude suggested CCA's built-in one
stays with compiler claude (it is part of the compiler) and Synthia goes
to a new session when it is built. Owner: "CCA's PM goes to compiler
claude, while Synthia goes to synthia claude." The Synthia session is not
started; its brief, clone and share of the ten cores (today 3 sessions ×
3 + 1) come when the owner starts it. `CLAUDE.md`, `compiler-session.md`.

### 2026-09-29 — Synthia is written in Go (owner)

The owner: "What should Synthia be in? I'm kinda feeling Golang." Rust
had been decided on 2026-09-14, when Synthia was expected to sit beside
the compiler and share its reviewers. Design claude noted the two-package-
manager split (below) left Synthia a separate tool that shares no code
with CCA: it reads `Synthia.ttsr`, fetches Limbs and runs `cca build …`,
which is Go's standard library (HTTP, archives, hashing, processes), with
goroutines for parallel downloads and one cross-compiling binary; the
costs are a second toolchain and the old "one language for reviewers"
reason, which assumed a team the owner no longer plans. Owner: "yeah,
Synthia in Go." The compiler, CCA's built-in package manager, the oracle
and the verification tools stay Rust. Who writes it is open (next).
`implementation-languages.md`.

### 2026-09-29 — One lane of a vector can be changed: `change 'v'[3] = 30;` (owner: a)

Compiler claude asked: `simd.md` had the lane read `'v'[1]` but no write;
both engines refused it. Proposed (a) a lane write like a list item, a
position outside the vector a check site; (b) a vector only changes by
being rebuilt (`simd.fill`, `simd.load`, `simd.shuffle`). Design claude
recommended a. The owner asked the performance difference (a lane write is
one instruction, `INS` on ARM; every workaround is the same or slower, a
round trip through memory about ten cycles; lane-by-lane writes in a loop
are slow in any language) and chose a. `examples/simd_lanes.cpls`; reject
test simd_lane_write_range.

### 2026-09-29 — A vector's lanes are checked at every operation, at the lane's type (owner: b)

Check claude asked whether lane arithmetic is exact in between like plain
numbers (2026-09-23) or checked per operation: `('a' + 'a') - 'a'` on
`simd[uint8, 16]` of 200s gave 200 in the oracle and stopped at the `+` in
CCA; `simd.md`'s "the scalar rule applied lane-wise" read either way.
Proposed (a) exact, which needs wider lanes (a `uint8` sum in 16-bit
lanes, half as many per register); (b) each lane operation checked at the
lane's type. Check claude and design claude recommended b: the plain rule's
reason ("a 16-bit and a 64-bit add cost the same") does not hold for
vectors, and exact maths is a plain loop away. The owner asked "what",
then chose b. The oracle now checks each `+ - x xx` lane at the lane type;
the wrapping forms stay defined. Reject test simd_lane_step.

### 2026-09-29 — `CCA.cplr`: one plain `const` per setting, shapes from a built-in `cca` Limb (owner)

```
const 'name' := "shop";
const 'version' := "1.0.0";
const 'dependencies' := [cca::Dep[name = "geometry", version = "1.2"]];
const 'target' := "aarch64-macos";
```

- **One `const` per setting** (owner: a). Proposed alternatives: one
  struct holding everything (b); a block of its own mini-syntax (c),
  which would not be CyborgPL syntax. Every name and type is checked
  against the settings CCA knows, so a misspelling is an error.
- **The shapes come from `cca`, a tiny built-in Limb only `CCA.cplr` can
  reach** (owner: a), reached with `::` like `std::`. The owner asked
  whether it was `cca::Dep` or `cca.Dep` (design claude's sketch had
  `cca.Dep`, wrong: `::` crosses into another Limb), then whether `cca` was
  a Limb. Proposed alternative: the shapes written bare in a `.cplr`
  file, a name from nowhere.
- **Plain `const`, not `pub const`** (owner). The owner asked which was
  correct: both are legal; `pub` is for other Limbs, and the compiler
  reading the file is not one.
Open: the full list of settings, and what a `cca::Dep` holds (where a
dependency comes from). `tooling.md`.

### 2026-09-29 — Synthia's manifest is `Synthia.ttsr`, in a syntax of its own (owner)

Following the two-package-manager split (below), the owner said Synthia
"will likely not be TOML inside", asked how an external package manager
connects (it reads its manifest, fetches the Limbs, and calls CCA with
where each is; CCA never reads the manifest), whether Synthia should be
`.cplr` (only if CyborgPL syntax), then proposed `.sss` (*Synthia Special
Syntax*; taken by SugarSS, a CSS syntax in PostCSS/Parcel/webpack),
`.ttr` (*Tankun's TOML Ripoff*; used by TETR.IO replays), and `.ttsr`,
**Tankun's TOML Syntax Ripoff**, free everywhere: "yeah. And no, the
syntax may not be like TOML." Design claude had suggested keeping `.cplm`
for Synthia's own syntax. `.cplm` is retired; the extension family is
`.cpls` (source), `.cplr` (CyborgPL-syntax tooling files, `CCA.cplr`),
`.ttsr` (Synthia's manifest), `.limb` (a published Limb). The syntax
itself is open; Synthia is not being built. `tooling.md`, `limbs.md`.

### 2026-09-29 — Two package managers: Synthia, and CCA's own, whose manifest `CCA.cplr` is CyborgPL (owner)

The owner asked "who said TOML inside?" of `Synthia.cplm`. Design claude
checked: on 2026-09-23 it had bundled "the TOML-inside decision" into the
confirmation of the `.cplm` extension, and the owner's "`.cplm` it is,
yes" was recorded as choosing it; the alternatives (a format of our own,
the manifest in CyborgPL as `pub const`s) were never chosen between. Shown
the CyborgPL form, the owner: "Since, it's a hobby lang, and a hobby is
supposed to be fun. There should be two package managers. Synthia is the
not built-in one. Our built-in PM would have CyborgPL syntax deliberately."
- **Synthia** stays the separate tool. What is inside `Synthia.cplm` is
  **open again**: design claude read the split as keeping TOML there, and
  the owner corrected it: "Synthia will likely not be TOML inside." 
- **CCA has a package manager built in** (owner, asked "idk, what do you
  suggest?"; design claude suggested the compiler, which already parses
  CyborgPL and evaluates constants, and keeps 2026-09-23's "dependencies
  in the manifest, never in source"): `cca add …`. Proposed alternative:
  built into the language, the manifest an ordinary source file.
- **It has no name of its own**: "just CCA's package manager" (owner: a).
  Proposed alternative: a name of its own, as Synthia has.
- **Its manifest is `CCA.cplr`**, *CyborgPL Related*, written in CyborgPL
  syntax (owner). `.cplr` is the extension for CyborgPL-syntax files that
  belong to the tooling; the name says which file it is. Proposed: `CCA.cplm`
  (one extension, two formats), `CCA.cpls` (reads as program source),
  `CCA.cplc` (*CyborgPL Config*, the owner's own idea of 2026-09-23); the
  owner asked whether `.cpm` (*Cyborg Package Manager*) was taken (free as
  an extension; the name CPM belongs to CPM.cmake) and then proposed
  `.cplr`, free everywhere.
What goes in `CCA.cplr`, and how the two managers relate, is open.
`tooling.md`.

### 2026-09-29 — A `bin` never holds an infinity or a NaN (owner: a)

Raised by compiler claude: a `bin` infinity or NaN into a `deci` had no
answer (the oracle panicked; CCA stopped). Behind it: text never makes
one (2026-09-24), and a `bin16`/`bin32` past its largest is a check site,
but `bin64` overflow, `1.0 div 0.0` and `0.0 div 0.0` were unsaid, and the
oracle let them produce C's infinity and NaN. Proposed (a) never: an
operation whose result would be one is a check site, like integer
overflow and division by zero; (b) IEEE's way, like C, with NaN's
oddities (`NaN == NaN` false, spreading silently). Design claude
recommended a; owner: a. So a `bin` always holds a real number, and
`to.deci…[bin]` always has an answer. `NO_FUCKING_CHECKS` remains the
way to raw IEEE speed. Reject tests bin_infinity, bin_nan.

### 2026-09-29 — The prints are one family: `std::print`, `std::print.useful`, `std::print.format` (owner)

Raised by the `deci` conversion's useful form (below): the owner made
it `.useful`, and renamed the prints to match. `std::printu` is now
`std::print.useful`, and `std::printf` (2026-09-22) is now
`std::print.format`; the output stays last (`std::print.useful.stdout[…]`),
and the long alias of 2026-09-21 still names the useful print. Design
claude had called `print.format` new, forgetting `printf`; the owner
pointed it out. The old names are refused with the new ones. Every
example, test and note converted; `examples/print_vs_print_useful.cpls`,
`examples/print_format.cpls`; reject tests printu_renamed, printf_renamed.

### 2026-09-29 — `deci`: literals, rounding, and the conversion from `bin` (owner)

`deci32` / `deci64` / `deci128` (7, 16, 34 digits) had a type and no
behaviour; neither engine ran it. Three questions:
1. **A literal with more digits than the type holds is a compile error**
   (owner: b): `'q': deci32 = 3.14159265;` is refused, never rounded.
   Proposed alternative: round once, as a `bin` literal does.
2. **Each operation rounds to its width, IEEE's way, half to even** (owner:
   "IEEE for deci too"), as `bin` does. The owner asked what an explicit
   round would be (one call at the end, `std::deci.round[…]`, std's) and
   whether `bin` rounds by itself (yes, half to even). Proposed
   alternatives: half away from zero; an error whenever a result does not
   fit.
3. **`bin` → `deci`**: `to.deci64['w']` is the exact value rounded to the
   width, the stored-form convention; `to.deci64.useful['w']` is the
   shortest decimal that reads back as the same `bin` (owner: c, then
   `.u`, then `.useful`). Design claude first recommended the shortest
   form "as `bin` prints"; the owner pointed out that only `printu` prints
   it, and the default prints the stored form.
Then three more the same day:
4. **An integer that could have more digits than the `deci` holds converts
   with `otherwise`, failing `too_many_digits`** (owner: c): `to.deci64['c']
   otherwise …` for an `int64`; an `int32` into a `deci64` always fits and
   takes none. The owner asked what (a 20-digit `int64` into 16 digits),
   whether design claude knew what Precog was (the first answer had said
   "the program stops", which is only the working build), what the other
   `to`s do (`to.bin64[int]` rounds silently; `bin` → integer fails
   `not_whole`), what (b) would do, and whether `otherwise` was needed, which
   raised (c). Proposed alternatives: round, as `to.bin64` does (a); a check
   site like narrowing (b, design claude's first recommendation).
5. **Text with more digits than the `deci` holds fails `too_many_digits`**
   (owner: a), beside `not_a_number` and `out_of_range`; the grammar is
   `bin`'s. Proposed alternative: round.
6. **A `deci` keeps its trailing zeros, as IEEE does** (owner: a): `1.50`
   stays `1.50`, `2.50 x 2` is `5.00`, and `1.50 == 1.5`. `print` shows
   `1.50`; `print.useful` shows `1.5`. Proposed alternative: always drop
   them.
7. **Each machine lays a `deci` out its own usual way** (owner): DPD on IBM
   POWER and z, whose hardware uses it; BID elsewhere, as C's `_Decimal64`
   does there, so a `deci` passed to C always matches. Proposed
   alternatives: always DPD; always BID. The owner asked when
   hardware-compatible `deci` comes: using the decimal instructions needs a
   POWER or z target, compiler work for if-and-when.
8. **`deci` → integer fails `not_whole`, and `deci` → `bin` rounds**, as `bin`
   does (owner: "yeah, same as bin").
The oracle now runs `deci` (`examples/deci.cpls`; reject tests
deci_literal_digits, deci_int_needs_otherwise, deci_family_mix,
deci_widths_mix). The leftovers, the same day:
9. **`mod` takes the divisor's sign**, as integer `mod` (owner: a). The
   owner asked whether there is a `div.trunc` (for integers, std's; a
   `deci` `div` is ordinary division, so only a `mod.trunc` could apply,
   std's later). Proposed alternatives: the dividend's sign; no `mod`.
10. **`xx` takes a whole exponent and rounds like the rest** (owner: a).
    The owner asked whether `deci` rounds by itself (yes, question 2) and
    whether the others refuse (no: math rounds, numbers coming in refuse),
    and kept a. Proposed alternatives: any exponent; no `xx`; a check site
    when it does not fit exactly.
11. **A wider `deci` into a narrower one fails `too_many_digits`** (owner:
    a). Proposed alternative: round.
12. **A `deci` may be a range type, with `bin`'s rules** (owner: a). Design
    claude's example had written `deci64[0, 1]`; with `bin`'s rules the
    bounds carry a point, `deci64[0.0, 1.0]`.
13. **A `deci` power's exponent is an integer or a whole literal** (owner:
    a), raised by compiler claude: CCA refused a `deci` variable, the oracle
    checked it at run time. Proposed alternative: a `deci` variable, checked
    when it runs. Reject test deci_power_deci_exponent.
`examples/deci_more.cpls`; reject tests deci_fractional_power,
deci_narrow_needs_otherwise. `types.md`.

### 2026-09-29 — A generic `op`; one that overlaps a specific one is an error (owner: allow, a)

Open since 2026-09-20. Proposed (a) an `op` may be generic, like a `fn`;
(b) not, one per type or a generic `fn` by name. Design claude
recommended a, with the example `op ~= ['a': <T: Number>, 'b': <T>]`;
the owner asked where `Number` was declared (nowhere: an example, std's
interfaces are undesigned; declared by hand it lists `-` and `<`) and
allowed it. Then an overlap with a specific `op ~=` for `bin64`: (a) an
error, the 2026-09-23 two-meanings rule; (b) the specific one wins
(specialisation). Design claude recommended a (b lets a new `op` change
what old lines do); owner: a. A generic `op` may not use a language
operator's name (design claude's consequence). The oracle had run a
generic `op` though nothing decided it, and refused two `op`s of one name
for different types, against 2026-09-20; both fixed. `generics.md`;
`examples/generic_op.cpls`; reject tests generic_op_overlap,
op_same_types_twice.

### 2026-09-29 — `gives` is spelled `giveS` (owner)

While settling nested matches in `#strict`, the owner asked whether `give`
was already used (yes: it moves ownership; `gives` was a separate word)
and asked for alternatives. Proposed: `yield` (Java's switch), `is`
(recommended by design claude; the owner asked whether it could be
confused with `=`: not in position, though Python/C#/Kotlin readers know
`is` as a test), `value`, `result`. Owner: "`giveS` it is, yes, with the
capital `S`" — the word stays, and the capital keeps it apart from `give`.
The old spelling is refused with an error naming the new one. Reject test
gives_renamed.

### 2026-09-29 — A `match` alone in a case: the outer match decides (owner: a; `#strict`: yes)

Check claude found the engines split on a case whose block holds only
another `match`: a value (the oracle, which then refused the statement
match) or a statement (CCA). Proposed (a) the outer match decides; (b) a
`match` where a statement can go is always a statement; (c) a statement
`match` alone in a case ends with `;`. Check claude and design claude
recommended a. The owner asked what it is for (a choice inside a choice:
cancelled or paid, then pickup or post), what Rust does (the inner match
is the case's value, which must be `()` when the outer is a statement; a
`;` is the escape hatch), and whether not-a has any use (reading a match
alone, which `#strict` already gives), and chose a. For `#strict`, design
claude proposed that a bare nested `match` is always a statement, since a
value says `giveS`; owner: yes. The oracle had demanded `gives` there;
fixed. `enums.md`; `examples/nested_match.cpls`,
`examples/strict_nested_match.cpls`.

### 2026-09-29 — An argument takes its parameter's type as its context (owner: a)

The follow-up to the list-literal ruling. Design claude had written "a
call's argument gets no context" into that entry from check claude's
framing, and CCA had followed it (`f[0 -~ 1]` refused); the record was
corrected (912854a) and the question put. Proposed (a) an argument lands
in its parameter, so the parameter's type is its context, like a list
item and a declaration; (b) none, the argument says its size. Design
claude recommended a. Owner: a. `list.append`'s value takes the list's
element type. What a call returns still says nothing about its arguments
(2026-09-23). `operators.md`; `examples/argument_context.cpls`.

### 2026-09-29 — A borrow conflict is reported at the change (owner: a)

Check claude asked which line the error names when what a borrow borrows is
changed (or given away, or lent mutably) while the borrow is still used
later: CCA named the change, the oracle the later use. Proposed (a) the
change, with the later use in the message, as Rust does; (b) the later use.
Design claude recommended a (the change is the line to move or fix). The
owner asked why the program errors at all (the borrow is a bookmark into
the list; an append can move the list and free the old spot, which CCA's
binary really read before its fix) and chose a. The oracle now reports
at the change: "'s' is lent mutably here while 'r' still borrows it and
is used on line 25"; a list changed inside a `for … in lend` loop is
reported at the change too. `diagnostics.md`.

### 2026-09-29 — A list literal's items take the element type as their context (owner: a)

Check claude asked whether `'xs': list[uint8] = [0 -~ 1,];` wraps at 8
bits like `'x': uint8 = 0 -~ 1;`. The 2026-09-23 wording ("context stops
at a call, a conversion or a construction") could read a list literal as
a construction. Proposed (a) items take the element type, as a
declaration's value does; (b) the size stops at the brackets, and an item
that needs one writes it. Check claude and design claude recommended a.
The owner asked whether the `~` sat on the wrong side (no: `-~`, 2026-09-20),
for an explanation, how a list says its item type (`list[uint8]` on the
variable), confirmed `'xs': list[uint8] = [1, 2, 3];` is right, noted that
b would open a follow-up question (how an item states its size), and chose
a. Nested lists pass it down. Arguments were ruled next (entry above). The oracle had used the list type itself as
the items' context and refused `[0 -~ 'i',]`; fixed. `operators.md`;
`examples/list_item_context.cpls`.

### 2026-09-29 — `op and.then` / `op or.else` are refused (owner)

Handed over as a value, a function's arguments are worked out before it
runs, so `op and.then` could never skip its right side and would behave
as `op and`. Design claude proposed refusing both, with an error naming
`op and` / `op or`, which keeps to "the code isn't lying" (2026-09-25).
The owner asked "what" and, after a simpler explanation with `'n' !== 0
and.then 100 div 'n' > 2`, said "yeah, refuse them". This closes `op`:
the oracle now takes `op` before any operator. `functions-as-values.md`
section 4; `examples/operator_values.cpls`; reject tests
op_value_untyped, op_value_and_then, op_value_minus.

### 2026-09-29 — `op -` is subtraction; negation as a value is `op neg` (owner)

`-` is both subtraction and negation (the day's `neg` was dropped), so
`op -` was ambiguous. Design claude proposed letting where it goes
decide (a slot taking two numbers gets subtract, one number negate).
The owner proposed instead `op -placeholder` or `op neg`. A placeholder
such as `op -_` would give `_` a second meaning beside digit grouping;
design claude recommended `op neg`. Owner: "`op neg` it is". `neg`
exists only after `op`; in expressions negation is still `-'x'`.

### 2026-09-29 — Division is `div`; less/greater-or-equal are `</==` and `>/==` (owner)

The owner's own change, raised while listing the operators for `op +`:
"`/` becomes `div`, `-` becomes `neg` (so, `neg'x'`). Now, since `/`
can't be confused for `div` in our lang. We could change `>==` and `<==`
to `>/==` and `</==`". Design claude read the slash as "or" ("less /
equal") when restating it, and asked whether subtraction stays `-`
(owner: yes). Asked then how minus five is written, `-5` kept for
numbers typed directly (recommended) or `neg 5`; the owner: "Nvm, let's
just make it back to `-`, forget `neg`". So negation and subtraction stay
`-`, and only `div`, `</==`, `>/==` change. `div` is all division,
integers (floor) and bins alike; `std::div.trunc` keeps its name. `/`,
`<==` and `>==` are refused with errors naming the new spellings.
`operators.md`; every example and test converted; reject tests
slash_retired, lte_retired, gte (now naming `>/==`).

### 2026-09-29 — An operator is a value as `op +` (owner: a, `op +`)

Open since 2026-09-21. Proposed (a) a way to hand an operator where a
function goes; (b) none, write an inline `fn`. Design claude recommended
b (the inline `fn` is one line and always says its types). The owner
asked why an operator would be stored (mostly it is passed: a `fold`, a
sort's comparison, a calculator picking one at run time), asked for a
small program (`fold[lend 'xs', 0, op +]` against the inline `fn`,
checked in the oracle), and chose a. Spellings proposed: `op +` (the
word that defines one), `(+)` (Haskell, OCaml), `op[+]`. The owner
asked whether operators can be words (yes, 2026-09-20: `x`, `mod`,
`dot`), then "`op +` it is". The value's type comes from where it goes
(`fold`'s parameter, a typed declaration); `'f' := op +;` alone cannot
say which `+`, so it is refused (design claude's reading, not yet
ruled). `op and.then` / `or.else` and `-` were settled the same day
(entries above). `functions-as-values.md`.

### 2026-09-29 — `@x` on a returned borrow: required in `#strict`, optional elsewhere (owner)

Asked whether the group may be left out. The owner asked which has better
performance (neither: the group is checked at compile time and leaves
nothing in the program) and said "`-> lent str.utf8` is legal in normal"
files, where Precog works the group out from the body, as "optional,
verified" (2026-09-14) had it. Design claude recommended requiring it in
`#strict`, since a strict caller sees only signatures and the reader
should see what the result depends on. Owner: "yeah, required in
strict". A strict function returning `lent T` or `lent.mut T` without
`@x` is refused. `ownership-syntax.md`, `core-features.md`; reject test
strict_lifetime.

### 2026-09-29 — Handing back a borrow: `-> lent@x T`, with `@x` a lifetime group (owner: b, `@`)

Item 4, returning a function that borrowed something, turned out to be
covered by 2026-09-21 (a lend may leave its call, and Precog proves it).
The open part was the optional lifetime annotation, which `#strict` needs
since callers there see only signatures. Nothing said how a function
hands back a borrow at all. Proposed (a) name the parameters it borrows
from, `-> lent str.utf8 from 'a', 'b'`; (b) Rust-style group labels.
Design claude recommended a. Owner: b, "but what's the syntax?". Offered
`@` ("at"), `~` (already wrapping), and a Rust-style list up front.
Recommended `@` introduced by first use, like `<T>`. The owner asked
whether the group needs a name (yes, that is b's difference from a), what
follows `@` (the group's name, meaning only "same group"), and for a
comparison with Rust (`fn longer<'x>(a: &'x str, b: &'x str) -> &'x
str`), then "`@` it is". The oracle now returns borrows (it refused any
before) and refuses one from the wrong group, one of something dying at
the return, and a borrow returned without `-> lent` in the signature. Whether
the group may be left out was settled the same day (next entry up). `ownership-syntax.md`; `examples/lifetimes.cpls`; reject
tests lifetime_wrong_group, lifetime_local, lifetime_unsaid.

### 2026-09-29 — `for 'i', 'x' in lend 'xs'`: the position beside the item (owner: a)

Parked since 2026-09-17. Proposed (a) name both, the position first;
(b) leave it to a range loop and a read. Design claude recommended a,
since numbering and finding positions come up constantly. The owner asked
for an explanation (with `["ann", "bob", "cy"]` the block runs with 1/"ann",
2/"bob", 3/"cy") and chose a. The position is a `uint64`. A range with two
names is refused. With `perm`, both outlive the loop (oracle's reading).
`lists.md`; `examples/for_position.cpls`; reject test for_position_range.

### 2026-09-29 — A `match` gives a value: `{ "free" }`, and `{ gives "free"; }` in `#strict` (owner: a, b)

The first everyday item on the base-language list. The owner asked what
`match` is for (a branch per enum case, every case covered, the data
unpacked) and why the word (chosen 2026-09-20: matched against shapes;
`switch` has fall-through baggage, `when` reads as time). How a case gives
a value: (a) a block holding only a value is that value; (b) a word,
`empty { gives "free"; }`; (c) an arrow, `empty -> "free"`. Design claude
recommended a. The owner asked why `};` ends the line (the declaration
ends there, as after `5`). Owner: "`a` for normal, `b` for `#strict`".
Following the `pos` pattern, `gives` is strict-only and the bare value is
refused in strict files. In strict files statements may come before
`gives`. A case may leave instead of giving. The same rule gives
`otherwise` with several reasons a value. "`match` is a statement" had
been design claude's detail, not a ruling. `enums.md`, `core-features.md`;
`examples/match_value.cpls`, `examples/strict_match.cpls`; reject tests
strict_match_needs_gives, gives_outside_strict, match_value_unused.

### 2026-09-28 — `synthia tune`: CPU-or-GPU measured once, saved to `tune.cplt` (owner)

The owner asked whether Precog would run a loop on CPU and on GPU at
compile time, when the machine is the target, to see which is better.
Design claude: not inside a build, since Precog has no wall-clock (the
same source must give the same program) and CCA never runs what it
builds; the choice is a cost model. Proposed (a) the cost model only;
(b) the cost model plus `synthia tune`, which measures once on the real
machine and writes a file every later build reads, like PGO. Design
claude recommended b. The owner asked how it works (step by step: tune
runs both versions with given inputs, writes `tune.cplt`, builds read it,
stale or foreign entries are ignored with a note) and said "Yeah". Synthia
is the compiler session's; nothing here is in the oracle. `gpu.md`,
`reproducible-builds.md`.

### 2026-09-28 — A C variable holds C's plain number; `to.int32[Mode.vi]` converts back (owner: b)

Left open by the C-entry rule: `otherwise` follows a call, but a C
variable is read by its name. Proposed (a) `otherwise` after a C
variable's name, for C variables only; (b) a C variable of a C enum or
range type is refused, declared plain and converted with `to.Mode[…]`.
The owner asked for missed use cases. The one found was writing a C
library's mode variable, which b answers with a new never-failing
conversion from a C enum to its number, `to.int32[Mode.vi]`. Design claude
recommended b with that. Owner: b. Also fixed, the gaps compiler claude
found in the oracle: `ptr.read` of a C enum or range now takes `otherwise`
and fails `out_of_range` (it was refused with `otherwise` and accepted
without), and a C variable of such a type is refused. `ptr.mut` lent to C
means a list, array or text (a single variable can't be lent as a
pointer), so every element is checked. `ffi.md`;
`examples/c_variables.cpls`; reject tests c_var_c_enum,
ptr_read_c_enum_unhandled, enum_number_not_c.

### 2026-09-27 — `NO_FUCKING_CHECKS`: a fourth tier that trusts C's answers too (owner)

Out of the C-enum question: design claude sketched a stronger `nocheck`
that also skips the checks on numbers C hands back (placeholder
`nocheck.all`) and advised against it, since the check is a comparison or
two and guards against newer libraries. The owner: "I think there should
be a new tier", then "`NO_FUCKING_CHECKS` is it, no checks". It removes
what `nocheck` removes, plus the C-entry `out_of_range` checks. Like
`nocheck`, it goes before a statement or block, and is listed. Failures of
Cyborg's own operations are unaffected (design claude's reading, not
asked). The oracle treats it like `nocheck`, running the checks anyway and
reporting a vouch that fails ("`NO_FUCKING_CHECKS` at line N lied"); the
C side it can't run. `core-features.md`; `examples/no_checks.cpls`; reject
test no_checks_lied.

### 2026-09-27 — C's numbers are checked wherever they come in; callbacks take plain numbers (owner)

From compiler claude: the C-enum door was only at a call's return. A
callback's parameter, a C variable, `ptr.read` and a `ptr.mut` lent to C
could all bring in a number with no case, which is undefined behaviour in
a `match`. Proposed (a) checks only at doors, with plain numbers converted
by `to.Key[…]` elsewhere; (b) `nocheck` vouches for the number; (c) check
everywhere and stop the program. Design claude recommended a. The owner
asked for use-case differences: a newer C library adding a case is an
`out_of_range` under a and undefined behaviour under b. Then "could we do
a and b?", "why not b where a nocheck is?" (returns are `nocheck` calls
whose check stays, per 2026-09-22), "or another tier?", and "is `nocheck`
currently lying?". It isn't: `nocheck` removes check sites (bugs), and
this is a failure (an answer) that writing the type asks for. That gave
the rule: a C enum or range type is checked wherever it comes in from C
(return, lent pointer, `ptr.read`, C variable), failing `out_of_range`;
the plain type skips the check; a callback's parameter must be plain.
Performance: a comparison or two per value. Owner: "yeah, do that rule".
In the oracle: `to.Key[…]` and the callback refusal. The owner also said
"I think there should be a new tier", still to be asked about. `ffi.md`;
`examples/c_enum_convert.cpls`; reject tests callback_c_enum and
to_c_enum_unhandled.

### 2026-09-27 — `#strict`: `range[from 2, to 8, by 2]`, and the working build refuses too (owner)

The owner looked at loops (`range[2, 8, -4]` runs zero times) and said
that under `#strict` "the loop syntax should be more self-explanatory".
Proposed: labels on the bounds, `from` / `to` / `by` (reads as a
sentence) or `start` / `end` / `step`, and whether `by 1` must be
written. Design claude recommended `from` / `to` / `by`, with `by` always
written. Owner: that. Outside strict files the labels are refused, as
`pos` is. The same answer settled the working build in a strict file:
(a) an unproven site is refused in every build; (b) a run-time check as
elsewhere. Design claude recommended a. Owner: a. The oracle enforces the
labels, but not builds, which it doesn't model. Reject tests
strict_range, strict_range_by, range_labels_outside_strict.
`core-features.md`, `control-flow.md`.

### 2026-09-27 — `#strict` also writes out what Precog would infer; `nocheck` allowed in it (owner)

`nocheck` under `#strict`: (a) allowed, as a written and listed reason,
and needed for any C call; (b) refused, with such code moved to a
non-strict file. Design claude recommended a. Owner: a. The owner then
asked about "the syntax changes with `#strict`, like requiring `pos`".
Offered: 1 `pos` on every index, 2 parameter types, 3 return types, 4
`mut` on changed variables, 5 types on every variable, 6 `temp`/`perm`
on every `for`. Design claude recommended 1 to 4, and against 5 (the `:=`
pillar) and 6 (noise). Owner: 1, 2, 3 and 6, and "look at 4 again".
Offered 4a (`mut` on what changes), 4b (`mut` or `immut` on every
declaration), 4c (nothing, since `change` marks each change). Design
claude recommended 4a. Owner: 4b, "`#strict` gives a fuck about the reader
too". So in a strict file, `pos` comes back on every position (and only
there). Parameters and value-returning functions say their types, every
declaration says `mut` or `immut`, and every `for` says `temp` or `perm`.
All of it is in the oracle. `#strict` once at the top of a file is the
oracle's reading, not asked. Still open: the working build in a strict
file. `core-features.md`; `examples/strict.cpls`; reject tests
strict_decl, strict_pos, strict_param, strict_ret, strict_for,
strict_top.

### 2026-09-27 — `#strict`: guardrails required, written in the same function (owner: a)

The owner: "I kinda want to add `#strict`", with no clash with comments,
which need a space. Asked what it does: "change the syntax to become more
verbose, but for a reason", then "guardrails required under `#strict`".
Design claude explained that guardrails are not required today: Precog
proves what it can by itself, and the final build refuses the rest until
a guardrail, a tighter type or `nocheck` settles it. Proposed for what
counts as a guardrail: (a) the reason must be written in the same
function (an `if`, a range type, loop bounds), with Precog still checking
it but not using facts from callers or other bodies; (b) an `if` in front
of every site. Design claude recommended a. Owner: a. It is a file-wide
switch, which the 2026-09-14 "tiers are never global" rule's wording
forbade, but it only tightens. Open: where it goes and what it covers,
`nocheck` under it, and the working build. The oracle cannot check
proofs, so it only accepts the line. `core-features.md`.

### 2026-09-27 — `pos` is removed: `'xs'[2]` (owner)

```
'xs'[2]                         # was  'xs'[pos 2]
change 'xs'[2] = 5;             # was  change 'xs'[pos 2] = 5;
list.slice[lend 'xs', 2, 4]     # was  list.slice[lend 'xs', pos 2, pos 4]
simd.load[lend 'xs', 'i']       # was  simd.load[lend 'xs', pos 'i']
```

The owner: "Feeling like removing `pos`". Design claude laid out its two
jobs. The owner's own 2026-09-17 reason was a position against a value
(`2` as "the second" or "2"). Since 2026-09-21 it had also told a list read
from a call of a function held in a variable (`'f'[2]`). The owner asked
whether functions were bare (named ones are, held ones are quoted), then
"they could just add comments". While asking what
`simd.load[lend 'xs', pos 'i']` does, the owner read `'i'` as an item
(`'i' = 'xs'[5]`) rather than a position, the confusion `pos` was made to
prevent, and design claude said so. Owner: "Remove `pos`". Every `pos`
goes. `'x'[…]` reads a list when `'x'` holds one and calls a function when
it holds one. A list read alone as a statement is refused, and so is
`otherwise` on one. `pos` is no longer a keyword. Old spellings are refused
with "`pos` is gone". The oracle, examples, reject tests (pos_removed
replaces index_without_pos, simd_load_needs_pos, slice_needs_pos) and
`design/` are converted. CCA and check claude have not been told.

### 2026-09-27 — Oracle bug: `bit` was reserved, though no decision reserves it (compiler session)

The oracle refused `fn bit […]` as a keyword. `bit` only ever follows a
dot (`and.bit`, `or.bit`, `not.bit`), where a word needs no reserving,
like `.then`, `.some`, `.slice`, and `syntax-sketch.md`'s keyword list
never had it. Removed from the oracle's list. CCA already accepted it.

### 2026-09-27 — `simd.shuffle['v', [4, 3, 2, 1]]` rearranges lanes (owner: a)

The last SIMD leftover. Proposed (a) one general shuffle whose order is
fixed lane numbers known when compiling; (b) leave it, since the folds
cover the common cases. Design claude recommended a. Owner: a. Lane
numbers are 1-based, repeats are allowed, and a lane that does not exist
is an error. Two-vector shuffles (interleave) were not asked. `simd.md`;
`examples/simd_shuffle.cpls`; reject tests simd_shuffle_not_fixed,
simd_shuffle_lane. That closes the SIMD list in `simd.md`.

### 2026-09-27 — A simd value may be a struct field or a list element (owner: a)

Proposed (a) anywhere a number goes, a plain fixed-size value aligned to
the register width; (b) local only. Design claude recommended a. Owner:
a. The oracle already allowed it, with a simd value plain and copying by
itself, so only the record changed. `simd.md`;
`examples/simd_fields.cpls`. Asked in passing: `range[2, 7, -4]` runs
zero times, since counting down from 2 never reaches 7, and that is not
an error.

### 2026-09-27 — `simd.load.some` / `simd.store.some` for a loop's leftover end (owner: a)

A vector of N lanes over a list of any length leaves up to N−1 items,
which needed a second, plain loop. Proposed (a) loads and stores of only
the first k lanes, the other loaded lanes zero, with k items that must be in
the list (a check site); (b) leave the tail a plain loop. Design claude
recommended a, with the names flagged as placeholders. Owner: a, names
unchanged. The oracle's `simd.load.some` takes its lane count from the
declared type, as `simd.load` now does. `simd.md`;
`examples/simd_tail.cpls` (1 to 10 sums to 55); reject test
simd_some_past_end.

### 2026-09-27 — `simd[bool, N]` is a type you can write (owner: a)

A vector comparison already gives a `simd[bool, N]`, but the type could
not be written, so no function could take a mask. Proposed (a) a
writable type, with `simd.fill[true]`, `all`/`any` and lane-wise
`not`/`and`/`or`; (b) a mask only as a comparison's result. Design claude
recommended a. Owner: a. A mask has the lane count of what it compares:
2, 4, 8 or 16 on a 128-bit target. The oracle had allowed only 16, as if a
bool were 8 bits. Found on the way, an oracle bug: `simd.load` and
`simd.fill` always made 16 lanes whatever the declared type, and the
2026-09-23 context-stops-at-a-call fix had also cut them off from it. Both
now take the declared lane count. `simd.md`; `examples/simd_mask.cpls`;
reject test simd_mask_lanes.

### 2026-09-27 — A range type may be a `layout.c` struct's field (owner: a)

Refused until now (`input.md` Open). Proposed (a) allow it: C sees the
plain number, and the range is checked at the door when the struct comes
from C (`out_of_range`, as for a ranged return) and proven where Cyborg
fills it in; (b) keep refusing it. Design claude recommended a. Owner: a.
A range on an `extern fn` parameter is still refused (not asked).
`ffi.md`; `examples/c_struct_ranges.cpls`; reject test
c_struct_range_out.

### 2026-09-27 — `layout.c enum`: C's enum, a number per case (owner: a)

The first low-level item (the owner: "I'll do the low-level stuff first").
C libraries use enums constantly (raylib's keys: `KEY_A = 65`). Proposed
(a) `layout.c enum Key { a = 65  b = 66 }`, a Cyborg enum laid out as C's
number, whose cases carry no data, with a number C returns checked at the
door (`out_of_range`); (b) no enums at the boundary, plain numbers and
named constants. Design claude recommended a, with every number written
out. Owner: a. The enum is owned like every enum (2026-09-20), 32 bits,
and only a `layout.c enum` crosses to C. `ffi.md`; `examples/c_enum.cpls`;
reject tests c_enum_number_missing, c_enum_same_number, c_enum_data,
enum_not_c.

### 2026-09-27 — N% is renamed Lunae (owner)

"`Lunae` is the `N%`'s new name. I liked Lua's name." The scripting
language on top of CyborgPL (2026-09-18) keeps everything else. Its logo,
a full moon in nebula colours, is in `brand/lunae/` (f842f7c, from the
website session). The compiler's name, Nowork, is unchanged.
`scripting.md`, `tooling.md`.

### 2026-09-27 — `design/` rewritten to the 2026-09-25 syntax (owner: "do 2")

Every code sample and every sentence about declarations in `design/`
now uses the 2026-09-25 syntax: no `let`, `change`, `!$`, `!mut`, `extern
'x': T`. History sections keep the old form on purpose, and `DECISIONS.md`
keeps each entry as it was written. The same day, with the owner's
go-ahead, compiler claude, check claude and the website session were told
about the new syntax and `and.then` / `or.else`.

### 2026-09-25 — `and.then` / `or.else`: stop once the answer is known (owner)

Straight after choosing b (both sides always run), the owner asked for a
familiar `and` by the dot rule. Design claude proposed `and.then` /
`or.else`, as in Ada, which made the same split (plain `and` runs both,
`and then` stops early), against `and.guard`, `and.early` and `and.short`.
Recommended `and.then` / `or.else` for reading as English. The owner asked
what describes it best (these), confirmed `or.else` narrows `or`, asked why
`and.then` too (the everyday check-then-use is an `and`), what `or.else`
does, and why not `otherwise`. `otherwise` handles failures, and a list
read out of range or a division by zero is a check site, not a failure. A
failing `list.get` was offered for the list case. Owner: "Add the and and or
stuff": `and.then` and `or.else`, not `list.get`. They are in the oracle,
with Precog's narrowing left to CCA. `examples/and_then_or_else.cpls`;
`operators.md`.

### 2026-09-25 — `and`/`or` always work out both sides (owner: b)

From check claude: nothing said whether `and` stops early. Both engines ran
both sides, so `if 'n' > 0 and 100 / 'n' > 2` stops when `'n'` is 0.
Proposed (a) stop early, as nearly every language does, so the one-line
guard works and Precog learns from the left side (recommended by check
claude and design claude); (b) always run both, today's behaviour; (c)
run both and refuse a line whose right side can fail when the left
already decides. The owner asked whether Precog could help (it can prove
only what the program does, so under b it refuses the line), for use cases
of running both (every check reporting its own error, constant-time
comparisons), and walked through a list-index guard in CyborgPL and in
Python, where `&` shows b's crash. Then: "so the code isn't lying?". Under
b every piece written runs, and nothing is skipped silently. Owner: b. A
check-then-use is two `if`s. Nothing changes in the oracle or CCA, which
already did this. `examples/and_runs_both.cpls`, reject test
and_does_not_guard. `operators.md`.

### 2026-09-25 — `!$` in front: a name that cannot be shadowed; `immut` only forbids `change`; `!mut` = `immut` (owner)

```
immut 'a' := 1;   'a' $:= 2;      # allowed now: shadowing changes nothing
!$ 'c' := 1;      'c' $:= 2;      # error: 'c' cannot be shadowed
!$ mut 'total' := 0;              # changeable, never shadowed
!mut 'b' := 3;                    # the same as immut 'b' := 3;
```

The owner asked why `immut` could not be shadowed (the 2026-09-14 rule,
recorded without a reason) and whether it made any safety or performance
difference. None: a shadow is a second variable, and the old value is never
touched, so the rule only guards the reader. The owner split the two
promises: `immut` keeps "cannot be changed", and a new marker says "cannot
be shadowed". Design claude proposed `sealed`, `final`, `fixed`, `pinned`,
and recommended `sealed`. The owner asked why not `!$`. Design claude
argued `!` isn't "not" in Cyborg, which was wrong: `!==` is exactly that
pattern, so it counts for `!$`. The owner found `'b' !$:= 1;` unreadable
("looks like a stroke") and moved it in front of the name, where `mut` and
`immut` go. Owner: `!$` in front, and `!mut` as another spelling of
`immut`. `!$` combines with `mut`/`immut` (`!$ mut 'x'`). A loop's counter
stays unshadowable, as before. `!$` on a declaration that itself shadows
(`!$ 'x' $:= …`) is allowed and not ruled on. Earlier the same day the
owner considered `$` as an alias for `nocheck` and dropped it on the
2026-09-19 no-alias rule. `examples/sealed.cpls`; the reject test
shadow_sealed replaces shadow_immut.

### 2026-09-25 — No `let`: a declaration starts with its name; changing says `change` (owner)

```
'x' := 1000;            # was  let 'x' := 1000;
'x': int32 = 1000;      # was  let 'x': int32 = 1000;
mut 'x' := 1000;        # was  let mut 'x' := 1000;
'x' $:= 2000;           # shadowing, was  let 'x' $:= 2000;
change 'x' = 2000;      # changing, was  'x' = 2000;
change 'xs'[pos 1] = 5;
change 'p'.x = 3;
extern 'errno': int32;  # C's variable, was  extern let 'errno': int32;
```

The owner asked whether `'x' := 1000;` was ambiguous. It was not: the
oracle refused it as a missing `let`. The owner wanted it as the
declaration. Design claude noted that the Go pitfall (`:=` silently making
a new variable) cannot happen here, since redeclaring a visible name needs
`$`, and that the cost is scanning for declarations and churn. The owner:
drop `let` from every declaration, and use a word for changing a value if
declaring and changing would clash. For the compiler they don't (what
follows the name tells them apart); for the eye `'x': int32 = 1000;` and
`'x' = 1000;` are close. The owner wanted a word anyway, one saying it
cannot declare. Proposed: `change`, `update`, `set` (in Tcl and shell `set`
creates), `now`. Design claude recommended `change`. Owner: `change`. This
reverses the 2026-09-19 `let`. `let` and a bare `'x' = …;` are refused with
the new spelling. `extern let` became `extern 'x': T;`, which follows from
"all of them" but was not asked separately.

Only the oracle changed so far (owner: "only has to go in oracle for now"):
the parser, all examples, the reject tests, and new reject tests
let_removed and assign_needs_change. Not yet updated: `design/` prose and
samples, and CCA, check claude's generator and the website grammar, which
all still expect `let` until the owner tells those sessions.

### 2026-09-24 — Oracle messages: two told the wrong story (compiler session's phrase review)

Compiler claude re-reviewed every CCA reject phrase against its `.err`,
after the owner's re-review rule. Two oracle messages turned out to be
parser accidents, not the rule. `lang:let` said "a call needs `[…]`", and
now says `lang:` was removed (2026-09-14) and keywords are bare. `true[]`
said "expected `;`", and now says bare `true` is the boolean, and that a
function named `true` is reached by path, `limb.true[…]`, per 2026-09-19's
rule that a keyword may name a function. CCA's phrase for the second
("`true` … cannot name a function") contradicts that rule, since
`fn true` is legal. Reported to compiler claude. `examples/keyword_fn.cpls`
now shows `limb.true[]`.

### 2026-09-24 — The Unicode version is pinned, and updated deliberately (owner)

From compiler claude: grapheme boundaries shift between Unicode versions,
and CCA had pinned one (Unicode 17.0.0, unicode-segmentation 1.13.3) without a
ruling. Proposed (a) pin it per build, recorded in the lock file; (b) follow
whatever the compiler ships. Design claude recommended a, per
`reproducible-builds.md`. Owner: "Pin it, but we should update when we
should update". So it is pinned, and moving to a newer version is a
deliberate update made when one is worth adopting. Found alongside: the
docs had called "เกี่ยว" two graphemes. Under UAX #29 it is four
(เ | กี่ | ย | ว), and was corrected in fdf9b60.

### 2026-09-24 — cyborgpl.dev is live, as a placeholder; the real site waits (owner)

The owner had the website session put up cyborgpl.dev: one plain page,
"CyborgPL is under development.", linking to the GitHub repo. The repo is
still private, so the link fails for visitors, and the owner knows. The
owner's choices, relayed by the website session and confirmed here:
hosted on Cloudflare's free tier, as Workers with static assets (the
website session recommended them over Pages: the domain lives in config,
and nothing deploys on push); `www.cyborgpl.dev` redirects to
`cyborgpl.dev`, keeping path and query; the Workers are named
`cyborgpl-website` and `cyborgpl-www-redirect`; and the real website is
"not building the website for now". Deploys are manual and happen only
when the owner asks. The config and the page are in
`website/placeholder/` (aac7f14).

### 2026-09-24 — A lend's sizes must match the parameter's (owner: a)

From compiler claude: `total[lend 'big']` with `'big'` a `list[int64]` and
`total` taking `lent 'xs': list[int32]`. The oracle accepted it, since it
did not model widths, and CCA had compiled it reading the wrong layout.
Proposed (a) a type error; (b) a converted copy, each element a narrowing
site, written back for `lend.mut`. Design claude recommended a, since
`lend` is the word for no copy. Owner: a. Range bounds do not change the
layout, and a `:=` collection or a generic's `<T>` is not compared. The
oracle refuses the mismatch (reject test lend_width_mismatch).
`ownership-syntax.md`.

### 2026-09-24 — A function carrying a `copy`/`give` capture is owned (owner: a)

From compiler claude, building functions as values: can
`fn […] with [copy 'xs']` be copied by `let 'g' := 'f'`, and who frees
the captured list? Proposed (a) a function that captured by `copy`/`give`
is owned and takes verbs like a list, freeing its captures when dropped,
while one carrying nothing, or only lends, stays plain; (b) every function
value takes verbs; (c) refuse `copy`/`give` captures, undoing 2026-09-21.
The owner asked what c does: only borrowing functions, none that outlives
its data. Design claude recommended a. Owner: a. The oracle had let `'f'`
copy itself silently, sharing the captured list. It now refuses the bare
form, and `copy 'f'` copies owned captures and keeps lends as lends.
`examples/owning_function.cpls`, reject test owning_function_bare.
Compiler claude's provisional layout (two pointers, code and captures)
is its own pick.

### 2026-09-24 — Oracle bug: text into a bin16/bin32 was still rounded twice (compiler session)

8705014 fixed literals, but `to.bin16["…"]` and `to.bin32["…"]` still read
text at 64 bits first. `"65519.99999999999999999"` into bin16 failed
`out_of_range` (correct: 65504), and three more cases were one step off.
Fixed: text is rounded once from its digits, with an exponent spelled out
before an exact halfway point is decided. `examples/bin_widths.cpls`.

### 2026-09-24 — Which text is a bin: digits, fraction, exponent; no `inf`/`nan` (owner: b)

From compiler claude: the oracle took whatever Rust's float parser takes
(`1e3`, `.5`, `inf`, `infinity`, `nan`), and nothing had chosen it.
Proposed (a) the source grammar of a bin literal; (b) that plus `e`
exponents, common in data files; (c) the oracle's behaviour. The owner
asked whether it needs `otherwise` (yes, always: it can fail
`not_a_number` or `out_of_range`), then whether to accept c too. Design
claude advised against it: `nan` is not equal to itself and would break
comparisons, sorting and map keys, and source code cannot write either.
Also offered (d), recognising them to fail with the right reason. The
owner asked what happens when a data file just contains `nan`: it lands
in the `otherwise`, which says what "missing" means there, and no NaN
ever reaches a sum. Owner: b. Too big (`"1e400"`) fails `out_of_range`,
per the ruling earlier today. The oracle's picks for a leading `+`, `_`,
`.5`/`5.` and spaces are provisional. `examples/text_to_number.cpls`.

### 2026-09-24 — Oracle bug: a bin literal was rounded twice (compiler session)

`let 'x': bin32 = 340282356779733661637539395458142568447.0;`, just
under bin32's rounding limit (2^128 − 2^103), was refused. The oracle
read every literal at 64 bits first, which landed exactly on the limit
(a tie, rounding to infinity), and then rounded again. The same double
rounding can move an ordinary literal one step near a halfway point. CCA
rounds once, from the digits (LLVM's APFloat), as C compilers do. Fixed:
a literal landing in a bin16/bin32, or meeting a bin16/bin32 value in
arithmetic, is rounded once from its written digits (bin32 by a correctly
rounded parse, and bin16 with the digits deciding an exact halfway
point). A literal in a comparison is left at 64 bits, as before and as in
CCA. `examples/bin_widths.cpls`, reject test bin32_literal_past.

### 2026-09-24 — `bin16`/`bin32` math rounds after every step (owner: a)

From compiler claude: `'x' + 'one' + 'one'` on `bin32` values of
16777216 and 1 gives 16777216 when rounded after each operation (IEEE at
the type's width, like C on ARM64), or 16777218 when done at 64 bits and
rounded once where stored (what the oracle did). `precog.md` ("IEEE bins
at their width") and `reproducible-builds.md` already leaned towards the
first. Proposed (a) round after every operation; (b) 64-bit intermediates.
Design claude recommended a: for bins, unlike integers, the wide
intermediate costs half the SIMD lanes and cannot run on Metal (no 64-bit
float), and bin16/bin32 are chosen for exactly that speed. The owner asked
whether a different type avoids it: yes, bin64 (exact to 2^53), an
integer, or `std::uenr`. Owner: a, "the programmer could've chose a better
one". A bare literal takes the width of the typed value beside it, the
destination never changes it, and a result rounding past the largest is a
check site. The oracle's pick for mixed bin widths (the wider) is
provisional. `examples/bin_widths.cpls` shows both sums; `operators.md`
has the rule.

### 2026-09-24 — Oracle bug: bin16 and bin32 were kept at 64 bits (compiler session)

`let 'f': bin32 = 0.1;` printed the 64-bit 0.1
(0.1000000000000000055511151231257827…), where the 2026-09-14 entry
gives this exact case as `0.100000001490116119384765625`. The oracle never
rounded a bin to its width. Fixed: a bin16 or bin32 is rounded
(IEEE, nearest-even) where it lands, on a typed `let`, an assignment, an
argument, a return or a field, and by `to.bin32[…]`. A value that rounds past the
largest (65520 into bin16) does not fit: a check site, or `out_of_range`
from text or into a range type. `printu` now shows the shortest text that
gives the value back at its own width (`0.1` for a bin32 0.1, per the same
entry). A `bin64` holding the same bits still shows `0.10000000149011612`.
Whether bin32 arithmetic also rounds after each operation is before the
owner and is not changed here: the oracle still computes an expression at
64 bits and rounds where it lands. `examples/bin_widths.cpls`, reject test
bin16_rounds_past.

### 2026-09-24 — Oracle bug: a huge shift count wrapped to a small one (verification session)

Generator seed 1125, shrunk: `(0 shr 4611686018427387904) > 0` ran in the
oracle and was refused by CCA, which was right. The shift-count rule
(0 to width − 1, a check site) compared the count only after narrowing
it to 32 bits, so 2^62 became 0. Fixed by comparing the whole count. A
shift with no width anywhere now says "on a value of any width" instead
of the oracle's internal 127 bits. Reject test: shift_count_huge.

### 2026-09-24 — Text too big for its number type fails `out_of_range` (owner: c)

From compiler claude: `conversion.md` did not say what a well-formed number
too big for its type does. The oracle trapped it as a narrowing site, and
past 2^127 it failed `not_a_number`, an edge nobody chose. Proposed by
compiler claude: (a) keep that; (b) always a narrowing site. Design claude
added (c) fail `out_of_range`, as a conversion into a range type already
does, and recommended it. Under (b) the final build could never accept a
program converting outside text, since a check site on outside text can't
be proven, and `input.md`'s rule is "a failure you handle, never a crash".
Owner: c. So text → number is `or fail[not_a_number, out_of_range]`, however
many digits, and a handler that lists reasons must list both. The owner
asked what happens with no `otherwise`: a compile error, as for every
failing call. Even passing it up is written, `otherwise fail`.
`examples/text_to_number.cpls`. The reject test to_unhandled now names
both reasons.

### 2026-09-24 — A negative exponent is a check site, for `xx` and `xx~` (owner: a)

From compiler claude, building powers at run time. `2 xx~ 'n'` with `'n'`
negative gives a fraction. `operators.md` said wrapping is "never a
check", which is about overflow, and nothing recorded exponents, not even
for plain `xx`. Proposed (a) a check site for both, like any unproven
safety property; (b) a fixed answer such as 0, so `xx~` stays
check-free; (c) the exponent must be unsigned. Compiler claude and
design claude recommended a. Owner: "big Precog sees the whole program,
and those vars already use `:=`", that is a. The oracle's refusal
became a check site (a `nocheck` over it is reported as a lie), and there
is a reject test, negative_exponent. Found on the way: the mixed-size
rule for wrapping refused `'h' xx~ 'n'` with a `uint32` base and an
`int32` exponent. The oracle now counts only the base's size for `xx~`,
reading the exponent as a count like a shift's. That is provisional and
listed in the oracle README.

### 2026-09-24 — Oracle bug: text too big for a range type's base crashed instead of failing (design session)

Found while checking a question from compiler claude. `input.md` says a
conversion into a range type fails `out_of_range`, "so a bad outside
value is a failure you handle, never a crash". But
`to.uint8[0, 100]["99999999999"] otherwise 0` crashed with "does not fit
uint8", because the base type was checked before the range. Fixed: into
a range type, a value that does not fit the base type fails
`out_of_range` too. `examples/ranges.cpls` pins it. Plain
`to.int32["99999999999"]`, with no range, is a separate question put to
the owner.

### 2026-09-24 — Type names need not be capitalised (owner)

Asked by the website session, whose syntax colouring treats a
capitalised bare word as a type: should type names be required to be
capitalised, as module names are required to be lowercase? Owner: no.
Nothing ever required it (`struct locker { … }` and `enum status { … }`
run in the oracle); this records that it was asked and answered, so
capitalised types stay a convention, not a rule.

### 2026-09-23 — Precog proves recursive functions by induction (owner: c)

The benchmarks were rewritten in proper CyborgPL, each stating its input
range where the input comes in. The final build then proved every site
in sum, lists and sieve, and refused fib's `fib['n' - 1] + fib['n' - 2]`
although `'n'` in `[0, 92]` makes it correct (fib(92) fits `int64`).
Proving it needs induction over the recursion, and Precog follows calls
a few levels deep. Proposed by compiler claude: (a) `nocheck` on that
line with a comment (recommended for now, by compiler claude and design
claude); (b) leave fib in the working build as a known limit; (c) teach
Precog to bound recursive functions by induction. Owner: c, "I can
wait". A Precog feature, `precog.md`'s calls rule.

### 2026-09-23 — The website's text is CC BY 4.0 (owner)

The owner chose it with the website session, which explained it
(attribution-only, commercial use allowed, irrevocable, no trademark or
patent rights) and relayed the choice. The owner confirmed it here:
"yeah, CC BY 4.0 for the text". `website/` code stays Apache-2.0 WITH
LLVM-exception, and the name and logo stay reserved. The licence entry
below had covered "everything that is code" and left prose unaddressed.
Only the website's text is ruled; `design/` and `DECISIONS.md` stay
under the repo's licence. `licensing.md` says so. This closes the open
line in the `website/` entry above.

### 2026-09-23 — The website lives in `website/` in this repo (owner)

The website session is building a new open-source site inside the
language repo, and asked where. Proposed: `website/`, `www/`, `docs/`,
`site/`. Design claude recommended `website/`, which says what it is and
leaves `docs/` for language documentation. Owner: `website/`. The site
is updated only when the owner asks the website session to. It becomes
public when the repo does (overtaken 2026-09-24: cyborgpl.dev is live, above).
The site's text licence was open here, and
is ruled above: CC BY 4.0.

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

### 2026-09-14 — Shadowing only with `$:=` / `$=`; never for `immut` (owner) — `immut` part replaced 2026-09-25 by `!$`

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

- SIMD: shuffling two vectors together (interleave). The other leftovers were decided 2026-09-27. (`design/simd.md`)


- GPU offload (owner chose automatic, 2026-09-21): the offload test, the runtime, target descriptions for GPUs, what the body may contain. Not designed.

**Language**
- Lunae (formerly N%): its syntax (undesigned), boundary defaults in the translation, translator verification. (owner's direction, 2026-09-18)
- Generic `const`; std's standard interfaces; associated types; an interface promising a `const` or field. (2026-09-20)
- Concurrency: cancellation by hand; `whichever` with `send`; oracle support. (2026-09-22)
- Conversion, still open parts: a useful-form (short) number → text. (2026-09-19)
- A per-parameter `bare` flag (arguments written without quotes) — sketched for `std::c.printf`, advised against, shelved. (2026-09-19)

**Precog**
- Non-terminating compile-time code: with step budgets now required, the budget's size and the error it produces. (Claude, 2026-09-14)

**Process**
