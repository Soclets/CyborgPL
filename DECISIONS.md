# CyborgPL v2 — Design decisions

Running log. Each entry: date, the decision, who proposed it, and who chose it.
Nothing here is final until the owner says so; open questions live at the bottom.

## Decided

### 2026-10-09 — The Nerve icon is official; tried at the README's top, then taken out (owner, via website claude)

Relayed by website claude with the owner's words: "Tell design claude to put
the icon in the README"; the icon itself, website claude's work (43ce7ebd):
"The last one, official now", and "The icon should be with its background,
not no-background". It is a starburst nerve (a ring for the cell body, a dot
at its centre, ten nerves radiating out, long and short in turn, each ending
in a bulb), always on its own black rounded tile: `brand/nerve/icon.svg`,
never the bare `mark-*.svg`. **The README's top shows the icon above the
NERVE wordmark** (72 px wide), one file for both themes since it carries its
tile; the README's `brand/` line names it. Design claude first put it on the
wordmark's left; the owner: "Format's broken in the README. Logo should be on
top, not on the left", then "Ahhh. It's look weird, remove it from there": **the
icon is not at the README's top**; only the wordmark is. The `brand/` line
still names the icon.

### 2026-10-09 — The names and logos are trademarks anyone may use, without pretending to be us (owner)

The owner asked whether the repo says what CC BY 4.0 covers (the README
does; the bare licence file, the README's warning line and GitHub's two
side-by-side licence tabs do not), then "What about our logo, does CC BY
cover that": no, CC BY is the website's text only, the name and logo were
reserved (2026-09-23), and the logo files fall under the repo's default
licence. Asked whether the files should stay under Apache-2.0 or be
excluded, the owner: "Mf, I don't have a lawyer. The logos should be
trademarked (not registered yet), but using it anywhere is possible, but
shouldn't pretend to be us". So **the names and logos are unregistered
trademarks (™); anyone may use them anywhere, but not to pass themselves
off as the project** (the official Nerve, or made or endorsed by it). Design
claude then wrote "the logo files stay under the repo's licence", its reading,
not asked; the owner: "So, our logos are Apache 2.0?" Offered Apache-2.0
(notice duty), CC BY 4.0 (credit duty) or CC0 (nothing), with design claude
leaning CC0; the owner: "Don't we already have CC BY?", then wrote the credit:
"Nerve logo(s): <official website>, <official GH repo>", and "yea" to
applying it, and changed the website text credit to match ("separate one,
go"). So **the logo files in `brand/` are CC BY 4.0, credited `Nerve logo(s):
https://nerve-lang.com, https://github.com/Soclets/Nerve-lang`**, and **the
website's text credit is `Nerve website text: https://nerve-lang.com,
https://github.com/Soclets/Nerve-lang`**, replacing `Original: Nerve™ (product
name) at (one of the product's locations) nerve-lang.com (official
website)`. The GitHub link 404s for others while the repo is private (the
owner kept it). The README, the scope note in `LICENSE-CC-BY-4.0` and
`design/licensing.md` say so.

### 2026-10-09 — The NERVE wordmark is the official logo (owner, via website claude)

Relayed by website claude, with the owner's words: "Mono as official logo.
Message design claude to add it to the README, replacing the original."
The logo is the NERVE wordmark (`brand/nerve/`, 3bffea57, website claude's):
heavy block letters with a nerve impulse cut through them at mid-height,
see-through, black (`wordmark-black.svg`) for light backgrounds and white
for dark. **The README's top shows it in place of the old C mark**, switching
with GitHub's light and dark themes, and its `brand/` line describes it. The
old C-mark files stay in `brand/` (not asked); the website keeps the old mark
until the owner asks website claude.

### 2026-10-09 — CCA is Synapse; the command answers to both `synapse` and `syn` (owner)

The last rename step, CCA → Synapse (chosen 2026-10-08): the owner, "Yea".
Asked for the command's name, (a) `synapse` or (b) an abbreviation (`syn`,
recommended, `snp`, `sy`), the owner: "Both". So **the compiler's command is
`synapse`, and `syn` is its short form**: `synapse build` and `syn build` do
the same; the old `cca` is refused, naming them. This is a deliberate second
spelling, the owner's, where 2026-10-07 made one spelling per thing the rule
in the language and `cca precog` was refused on it this morning. Then
**the folder and crates take the full name**: `synapse/`, `synapse-runtime`
and so on (owner: "Yeah"), `syn` being a typing shortcut only, as design
claude recommended. Compiler claude kept the `CCA_*` environment variables; asked, **they are
`SYNAPSE_*`** (owner: "`a`"): `SYNAPSE_TEST_JOBS`, `SYNAPSE_IR`,
`SYNAPSE_LLD_PREFIX`, and `CCA_STD_STUBS` becomes `SYNAPSE_CORD_STUBS`; an old
`CCA_*` name is refused, naming the new one. Check claude moved to `synapse
cortex --json` (9f3721ac), so `cca` can refuse. RRx2-TV, which reads
`cca/compiler/tests`, is not messaged; it reads its updated brief when the
owner next starts it (owner: "`b`").

### 2026-10-09 — Code ends `.nrv`, settings end `.nrp`; the manifest is `Kinesin.nrp` with `kin::` types (owner)

Before the endings step, design claude asked about the other `.cplr` files
(`tune`, `build`, `spellings`, `cpl_ver`, `installed`): (a) all `.nrv`, (b)
a separate ending, first recommending (a). The owner asked how TOML looks,
then "What is a broken `nrv`? Since some of those files aren't turing
complete": a `.cpls` is code, a `.cplr` one value; with one ending, what is
"broken" would depend on the file's name, and the compiler, editors and
the site would have to tell settings from code by name. Design claude
changed to (b). The owner: "I'm trying to be in theme, what do you call a
broken nerve?": neurapraxia (a nerve intact but not firing), palsy,
neuroma, neuropathy. Checked: `.nrp` overlaps only ASP.NET Report Maker's
old reports; `.palsy` with nothing, but reads as mocking cerebral palsy.
The owner: "yea". So **`.nrv` is code and only code, and a file that is one
Nerve value and never runs ends `.nrp`** ("neurapraxia": whole, but does
not fire), **the manifest included: `Kinesin.nrp`**, replacing yesterday's
`Kinesin.nrv`. Asked then whether the manifest's `cca::` types follow:
(a) now, as `kinesin::`, recommended, or (b) as `synapse::` with CCA → Synapse;
the owner: "`a`, but maybe use an abbriviation", then "yea" to **`kin::`**
("kin": a Neuron's relatives, its dependencies): `kin::Manifest`,
`kin::Lobe`, `kin::Dep`, `kin::Target`, `kin::Power`; `cca::` in a manifest is
refused as an old spelling, and `kin` is a reserved Neuron prefix. **Applied
the same day in the oracle and every example and test**: 1,029 files renamed
(`.cpls` → `.nrv`, `limb.cpls` → `neuron.nrv`, `CCA.cplr` → `Kinesin.nrp`), a
leftover old name refused naming the new one (three new reject tests), and
the notes changed to match (`tune.nrp`, `spellings.nrp`, `installed.nrp`).
Then, asked about the two left: **`build.cplr` is `build.nrv`**, a full
program (owner: "yea"; at a Neuron's root it is the build program, not a
module, in the oracle, design claude's reading), and **`cpl_ver.cplr` is
`nerve_version.nrp`** (owner's name, over design claude's `nrv_ver.nrp`).

### 2026-10-09 — The runtime is Axon (owner)

Next part name (chosen 2026-10-08): the owner, "Axon next". The runtime has
no word in programs and no name of its own in my areas, only CCA's crate
`cca-runtime` / `libcca_runtime.a` (compiler claude's, for the code-name
step). So **the notes call the runtime library Axon** where they mean it
(`parallelism.md`, `gpu.md`, `types.md`, `cores.md`, `licensing.md`,
`review-policy.md`, `compiler-session.md`, `implementation-languages.md`),
and `tooling.md` has an Axon entry. "Runtime" as an adjective ("a runtime
check", "at run time") stays, design claude's reading.

### 2026-10-09 — The base is the Brainstem (owner)

Next part name: the base, named Signatar on 2026-10-05 (its rename to "base
CyborgPL" was recorded on 2026-10-07 but never applied), is the Brainstem
(chosen 2026-10-08). The owner: "yea, go". It is never written in programs,
so **"Signatar" is "Brainstem" in the design notes, the oracle's comments and
messages ("Brainstem's names are English letters, digits and `_`",
"`inline.ASM` (Brainstem's)"), the oracle README, example comments and two
reject tests**; quoted words keep "Signatar", and `design/cores.md` says so at
its top. Not renamed, design claude's reading: the plain words "the base"
and "base" in prose, which are ordinary English as much as a name. Also
today: `cca precog` refused, naming `cca cortex` (99e3b6b2), after check
claude switched (44629ebd).

### 2026-10-09 — A core is a lobe: `lobes = [cca::Lobe[…]]` (owner)

Next part name, core → Lobe (chosen 2026-10-08). The owner: "Lobe next".
Design claude found the code words in every `CCA.cplr`, `cores =
[cca::Core[id = …]]`, and proposed `lobes = [cca::Lobe[…]]`, `cca::Lobe`
capitalized like `cca::Manifest` and `cca::Dep`; the owner: "yea, go". So
**a Neuron lists its lobes as `lobes = [cca::Lobe[id = "…", version =
"…"]]`**; `cores` and `cca::Core` are refused as old spellings naming the
new ones; the oracle's messages say lobe ("this Neuron's CCA.cplr does not
list the lobe ASP"), 80 manifests changed, two reject tests added. In
prose a core is a lobe, lowercase as "core" was; **CPU cores stay cores**
(the cores-per-session rule, "parallel on N CPU cores", the thread pool),
as do quoted words, "core features" and the file names `design/cores.md`
and `examples/slim_cores/`. Synthia's own `core.x.path` (paused) is left
for synthia claude.

The same message: "Don't message website claude, unless I tell you to.
Just website claude, the others are fine."

### 2026-10-09 — Precog is Cortex (owner)

Next part name, Precog → Cortex (chosen 2026-10-08: "`b`, only the people
with the link have seen it, and bots"). The owner: "yea, go". Precog is never
written in programs, so this is prose and messages only: **"Precog" is
"Cortex" in `design/`, the README, `CLAUDE.md`, the oracle's comments and
messages ("Cortex would refuse to compile", "`:=` means Cortex chooses the
type"), the example comments and one reject test**. Kept: quoted words,
`design/precog.md`'s file name, and the old name's history in
`design/tooling.md`. The README reads "Cortex™ (formerly Precog™)". CCA's
`precog` module and check claude's tools are theirs. Compiler claude renamed the command `cca precog` to `cca cortex`
(e36cb9a7) and kept `cca precog` as an alias for check claude's harness;
asked to (a) keep the alias or (b) refuse it like any old spelling, design
claude recommending (b) by the 2026-10-07 rule of one spelling per thing,
**`cca precog` is refused, naming `cca cortex`, and check claude switches**
(owner: "`b`"). CCA's internal `precog` module and `cca/cores/` crates keep
their names until the code-name step (compiler claude's call).

The same message: "Btw, didn't I say that only I could make website claude
update?" Yes (2026-09-24): design claude had asked website claude to switch
the site to Neuron after a "yea, go" that was about the repo (3f949b12, not
deployed). From now on design claude only informs website claude; the site
changes when the owner says so.

### 2026-10-09 — A library is a Neuron: `pub.neuron`; the root file waits for the endings (owner)

Next part name, Limb → Neuron (chosen 2026-10-08). Design claude found two
code words: `pub.limb` and the root file `limb.cpls`, and proposed
`pub.neuron` and `neuron.cpls`. The owner: "Isn't it `.nrv`?"; design
claude: yes, at step 3, so renaming the root file now would mean renaming
124 files twice. **`pub.limb` is `pub.neuron` now**, refused as an old
spelling naming it ("`limb` is an old spelling: write `neuron`"), and
**"Limb" is "Neuron" in prose and messages**; **the root file goes from
`limb.cpls` straight to `neuron.nrv` in step 3** (owner: "yea, go"). Kept:
owner quotes, `limb.cpls` and the `limbs/` folders until step 3, and
`design/limbs.md`'s file name. The README's trademark line reads "Neuron™
(formerly Limb™)", design claude's wording.

### 2026-10-09 — Rename step 2 begins: stdkit is `cord`, lowercase in code (owner)

Step 2 of renaming inside the repo, the part names, one at a time; design
claude proposed starting with stdkit → Cord (the Nerve name chosen
2026-10-08). The owner: "yea, go", then "lowercase syntax". So **the
standard library is `cord` in code**, `cord::print.useful[…]` as
`stdkit::print.useful[…]` was, and Cord in prose. `stdkit::` becomes an
old spelling, refused naming `cord::`, as `std::` was; `std` and `stdkit`
stay reserved as Limb prefixes. Design claude's reading, not asked:
`not_std.int24` stays, since it says "not standard", not "not stdkit".

### 2026-10-08 — CCA emits debug info (owner; not designed)

Asked whether the automatic locations bloat a program (no: compiler
messages are not in the program, a final build has no run-time checks,
only a working build's checks carry a location), then "what's the debug
info?" (DWARF / PDB tables mapping machine code back to lines, variables
and types, for debuggers and readable crash reports; often bigger than the
code, so usually stripped or kept in a separate file; CCA emits none
today, and nothing had been ruled). The owner: "There should be". So
**CCA emits debug info**. **A working build carries it inside; a final
build writes it to a separate file and the shipped program has none**
(owner: "`a`"), as macOS's `.dSYM`, Linux's split DWARF and Windows's
`.pdb`; over (b) every build carrying it and (c) working builds only.
Design claude recommended (a): the final product stays lean, and since
builds are reproducible the separate file always matches its program.
Not yet asked: the format per target (DWARF, PDB), and the file's name
and place.

### 2026-10-08 — A function can know where it was called from, as Rust's `#[track_caller]` (owner; not designed)

From "Do we need another IR for macros?" (no: macros work on tokens and the
AST, which CyborgPL has; Rust has no macro IR, its HIR/THIR/MIR coming
after expansion; CyborgPL has no macros), "What are macros for?" and "What
about the knowing the source?": built-ins already know it (TAS's `must`
names the line, values and operator; Precog's errors carry spans), but a
function the programmer writes cannot learn its caller's line, unlike
Rust's `#[track_caller]` (`Location::caller()`, how `.unwrap()` names the
line that panicked). The owner: "I want the track thingy". So **a function
can get the location it was called from**. **No mark: it is automatic**
(owner: "Shouldn't it be auto?", then "auto"): a function that asks gets
the location, and the compiler, seeing the whole program, passes it at
every call to that function; over (a) a mark as Rust's, which design
claude first recommended, then changed to auto. Design claude's reading,
not asked: a function called from C, or through a function value Precog
cannot resolve, gets "unknown" or is refused. **The location holds the
file, line, column and the calling function's name** (owner: "`c`"), as
`main.cpls:12:5 in parse_header`; over (a) file, line and column, which
design claude recommended (the names make a program bigger, which matters
in kernels), and (b) file and line. **Shown, it is two forms, as the
compiler's messages are (2026-09-22): one for humans and one for IDEs**
(owner: "shouldn't there be two location displays, the one for IDE, and
the one for humans"), **and the human one names everything the IDE one
holds** (owner: "Bro, with that, the human can't see where, if they read
human ver only" to design claude's first sample, which left out the file;
then "yea"): `called from parse_header, at line 12, column 5, in
main.cpls` and `main.cpls:12:5`. **The compiler's own human locations name the file too**
(owner: "yea"): every place a message names gives its file, so
`diagnostics.md`'s "called from MAIN at line 48" becomes "… at line 48 of
shop.cpls". Asked by compiler claude whether that reaches a message's
second places (a borrow "used on line 6", a case mark's "(line 2)", "the
first 'x' is declared here — line 4", `cca test`'s "FAILED: line 12"):
**always** (owner: "`a`", after seeing all three): "used on line 6 of
shop.cpls", "FAILED: line 12 of testing.cpls", even when the file was
named just above; over (b) never and (c) only when not already clear,
which design claude recommended. **It is in the base** (owner: "yea, base"): a location is
fixed text and two numbers, needing no heap or OS, and kernels report
where first (Linux's `BUG()` / `WARN()` with `__FILE__` / `__LINE__`);
only calls to a function that asks carry one, so a program that never
asks pays nothing (design claude's reasoning, recommended). Not yet asked:
the spelling (`caller.location` was design claude's placeholder).

### 2026-10-08 — A build may read environment variables (owner; how not yet asked)

The owner said "I kinda want include bytes, tbh"; design claude pointed out
that reproducible builds rule 3 (2026-09-14: "Precog never reads a file,
clock, or environment variable at compile time") forbids it, and proposed
an exception (files inside the Limb counting as source), not yet answered.
The owner asked why environment variables are banned (invisible inputs:
different per machine, unrecorded, can leak secrets), what they are for,
whether there is an alternative, whether `tune` replaces them (no: it only
holds loop timings), then for useful cases and today's alternatives
(version: a hand-written `const`; build ID, feature switches, C library
paths: nothing designed; target: `on.target`; secrets: read at run time).
The owner: "We should have env vars". So **rule 3 no longer bans
environment variables: a build may read them**. Not yet asked: how (which
variables, whether a Limb names them, whether the build records the values
it read so it can be repeated, whether a power gates it), and the
`include_bytes` exception. Asked how the build stays repeatable — (a) a
Limb names its variables in `CCA.cplr` and the build records the values it
read, recommended; (b) named only; (c) free — **any code may read any
environment variable while building, as Rust's `env!`** (owner: "`c`"),
knowing builds that read one are no longer repeatable from the source
alone and a secret read then can end up in the binary.

### 2026-10-08 — Precog's proof cache has a cache buster, and a final build may use the cache (owner: a; not built)

Talking about LLVM-sized programs under Precog, design claude recalled the
2026-09-16 allowance to reuse proofs for byte-identical unchanged code. The
owner: "Though, when we do caching, there should be a cache buster". So
**there is always a way to drop the cache and prove everything again**.
Asked whether a final build should always do that, **a final build may use
the cache like any other** (owner: "`a`"), over (b) every final build
proving from scratch; design claude recommended (a), since (b) brings back
the full cost on every release. Design claude's readings, not asked: the
cache key covers everything a proof depended on (the code, what it calls,
the compiler version and so the solvers', the target, the cores), the
buster being the way out when the key misses something; and since builds
are reproducible (2026-09-14), a cached and a fresh build must be
identical byte for byte, so comparing the two catches a cache bug (check
claude could run it). Not yet asked: how the buster is spelled.

### 2026-10-08 — CyborgPL links with LLVM's lld and never writes its own linker (owner)

The owner asked which linker CCA uses (the system one: `cc` with the object
and `libcca_runtime.a`, `cca/compiler/src/codegen/mod.rs`, from the
2026-09-14 pipeline sketch, never ruled on), why Zig's is praised
(cross-linking anywhere with one download), "Should we make our own
linker? I mean, Precog is already a thing we're working on", and "How many
LOC is a linker?" — counted by design claude on today's sources: lld
~100,700 lines of C++ for four formats (ELF 47,298, Mach-O 21,074, COFF
19,130, Wasm 10,396) plus ~181,700 of tests; mold ~41,600 lines of Rust for
ELF alone. Design claude recommended lld (all four formats, cross-linking,
linker scripts for kernels, part of LLVM, same licence) and no linker of
our own unless a need lld cannot meet shows up. The owner: "Oh wait, so we
won't have to make a linker? Just use lld?", then **"We'll use `lld`"**,
and on design claude's worry about reaching it through C++: "bro, C++ isn't
gonna make you guys have a stroke". lld has no C interface, so CCA reaches
it through the C++ LLVM wrapper `implementation-languages.md` already
allows. **CCA switches to lld now** (owner: "`a`"), so every build links
as the final one will and cross-building is ready when the rest is; over
(b) switching only when cross-building is first needed. Design claude
recommended (a). **lld is built into CCA, Zig's way** (owner: "`a`", asked
by compiler claude): CCA calls lld's library through a small C++ shim, so
lld is always inside `cca`; building CCA needs a C++ compiler, today on
Homebrew's lld dylibs as on its LLVM, and a self-contained release needs a
custom LLVM build for static libraries; lld's library is not made to run
twice in one process, which matters only if CCA links more than once per
run. Over (b) running whatever lld is installed and (c) shipping lld's
program beside `cca`, Rust's way (`rust-lld`), which compiler claude
recommended and design claude agreed with. Compiler claude checked first
that `ld64.lld` links a CCA object and the program runs as with `cc`.
Built the same day (39229d78): a 36-line shim; LLVM moved from static to
Homebrew's shared `libLLVM.dylib`, since lld's libraries load it (two LLVMs
in one process otherwise); a non-macOS host is refused for now. Told, the
owner: "I guess we'll do the other way when we do the release, or when
it's needed": **the static, self-contained build (LLVM and lld built by us
and copied into `cca`) waits for a release or a need**; Homebrew's shared
libraries until then.

### 2026-10-08 — If CyborgPL were renamed Nerve (a what-if, then likely): a library is a Neuron, the base the Brainstem, a core a Lobe, stdkit the Cord, the compiler the Synapse, Precog the Cortex, the runtime the Axon, the package manager Kinesin, files `.nrv`, the manifest `Kinesin.nrv` (owner)

Brainstorming names "in case I wanna rename" (owner: "CyborgPL is already a
good name, just finding more"), checked by design claude for clashes and
domains: Nerve was the front-runner (no language by that name; Elixir's
Nerves nearby; `nervelang.com` free), over Tin, Ion (Amazon Ion), Zenith,
Proof and others. The owner then found Nerve "not so fitting" and argued
for the current name: "It's human and machine, so you're mixing yourself
with the machine. … CCA isn't a submissive fuck like C, and it doesn't have
a GC either"; Nerve "is more like Rust, one syllable, and another way to
misread it". **CyborgPL stays.** Then, on "the feature names in Nerve":
syntax is never themed ("`no_check` is syntax, we don't theme that, bro"),
and Limb is the word for a library. **In Nerve, a library would be a
Neuron** (owner: "Neuron"), over Plexus (more accurate) and Ganglion
(accurate, clunky); the owner's bar: "catchy + kinda accurate is fine".
Design claude noted a neuron is closer to a function in size. Other Nerve
**The base would be the Brainstem** (owner: "Brainstem", after "Any alts?":
Stem, Reflex, Medulla, Spine), the lowest part, keeping you alive with
nothing above it, as the base runs with no OS and no heap. Other Nerve
names offered, not asked: Synapse (the linker; CCA calls the system
linker, so there is none to name), and, to Axon offered for the compiler, the owner: "Wouldn't that be the
runtime?". **The runtime would be the Axon** (owner: "`Axon`", after "Any
alts?": Impulse, Reflex, Myelin, Synapse, Signal), carrying the signal
while the program runs; today it is unnamed (`cca-runtime`). Later, the
owner: "CCA changes to Synapse instead of Cortex? And Precog becomes
Cortex?"; design claude agreed on Synapse (where code crosses from human
to machine) and recommended keeping Precog for recognition; the owner:
"`b`, only the people with the link have seen it, and bots". So **in
Nerve, the compiler is the Synapse and Precog becomes the Cortex**,
replacing the two choices above (CCA the Cortex; Precog keeping its
name). **In Nerve, a source file ends `.nrv`** (owner: "`.nrv` then"), after
design claude checked the others asked about: `.n` (Nemerle's source,
Neko's bytecode), `.nv` (old data formats such as MAME saves; reads as
NVIDIA), `.n1` (Envisat satellite data, an old virus generator; reads as
"version 1"), `.syn` (TextPad syntax files, chip synthesis; names the
compiler, not the language), `.nvr` (network video recorders); `.nrv`
overlaps only with Nero's old disc-compilation projects, and `.rs` has
such overlaps too. **The compiler's built-in package manager (CCA's today, `CCA.cplr`;
Synthia is separate) would be Kinesin** (owner: "Kinesin, it is"), the
motor protein that walks along an axon's microtubules carrying vesicles,
its "cargo", to the end; after Plexus (recommended first), Vesicle, Glia,
Dendrite, Ganglion, and the owner's "What's the thing that is the path,
the road for them" (Tract, Microtubule; Kinesin offered as what rides on
it, recommended). **The manifest would be `Kinesin.nrv`** (owner: "`a`"),
named after the tool that reads it, as Rust's `Cargo.toml` and today's
`CCA.cplr`, and written in Nerve itself; over (b) `Neuron.nrv`, named after
what it describes. **The compiler, CCA, would be the Cortex**
(owner: "Cortex is CCA"), the part that plans before anything moves, and
**Precog keeps its name** (owner: "yea, keep Precog"; Prefrontal and
Cerebellum offered). **stdkit would be the Cord** (owner: "`Cord`", after "Any alts":
Spine, recommended, Reflex, Senses, Plexus, Cortex): the spinal cord,
joining the brain to the body as the standard library joins the program to
screen, files and clock. **A core would be a Lobe** (owner: "yea"): each
lobe has its own job, as each core adds its own abilities; design claude
noted real lobes cannot be removed, while cores are optional.


Later the same day, the owner: "Also, we'll likely rename to Nerve. But
we'll do it cleanly, new GH repo, new local, but same code." So **the
rename to Nerve is likely, not done**: when it happens, a new GitHub
repository and a new local folder, with the same code. Nothing is renamed
or created yet. **The new repository keeps the git history** (owner:
"`a`"): the existing commits are pushed into it, so the commit IDs the
notes cite keep working; over (b) one fresh first commit. **The new
repository is `Soclets/Nerve-lang`, private** (owner: "yea, create it under
Soclets. `Nerve-lang`", over design claude's proposal of
`Cyborg-Assembly-Manufacturing-Site/Nerve`); design claude created it the
same day and pushed `main` (3d674c85) into it. The same day every session
moved to its clone of it (`/Users/ts/Nerve`, `… compiler`, `… verification`,
`… website`); the old repository was frozen at 087cdfc8, the old folders'
pushes disabled, and **the old repository archived** on GitHub (owner:
"yea, archive it"): read-only, kept, history intact. Then step 1 of renaming inside the repo (owner: "You could do only step 1
for tonight, tell the others to hop off"): **"CyborgPL" became "Nerve" in
the prose of `design/`, the example comments, the oracle's comments and
messages, and the README**. Kept as they were: words in double quotes (the
owner's and others'), the old v1 language ("legacy CyborgPL",
`CyborgPL-legacy`), what `.cpls` / `.cplr` / `.cplm` abbreviate ("CyborgPL
Source"), the install path `%APPDATA%\CyborgPL\`, the session titles, this
log (history), `bench/`, and every licence and trademark statement
(`design/licensing.md`, the README's "CyborgPL™" lines), which waited for
the owner. Then the owner: "Change the trademark shit". **The trademark
and licence wording names Nerve**: the README's heading, credit line and
attribution example (`Original: Nerve™ … nerve-lang.com (official
website)`), "Nerve™ (formerly CyborgPL™), Precog™ and Limb™ are names of
this project", and `design/licensing.md` throughout. Precog™ and Limb™
keep their names until the part names are renamed (step 2). Old clone paths in the session briefs now name the Nerve folders. Not yet asked:
whether names inside the code (CCA, stdkit, Limb, core) change
with it to the Nerve names above; the domain; the other sessions'
clones.
### 2026-10-08 — Xerpia is dropped: Precog is the compile-time language (owner: yea)

The owner, after the talk turned to the name's third reason ("multiple ways
to do a thing"): "Honestly, I might just drop Xerpia. A good name, cute, but
don't think there's much purpose in a language where Precog is the
language". Design claude agreed and named what would go with it, reading
files while building (`compile_time.io.read`) and writing code
(`compile_time.emit`), since Precog only works out values; generics cover
most of the second. Design claude recommended dropping it, nothing being
built; the owner: "yea, drop it". **Xerpia, the sub-language and its core,
is dropped**, and everything decided about it in the entry below with it
(`inline.Xerpia`, manual memory, `#bad_girl`, `bin16 %^x`, the `^` ladder,
`@` and `%`, and the public levels `@(^)`, which were agreed but never
recorded). The name stays the owner's, free for later use. Design claude's
reading, not asked: the `compile_time` power and its narrowings were for
Xerpia and go with it; reading files and writing code at compile time
could come back later as powers on ordinary CyborgPL run while compiling
(design claude's suggestion, not asked).

### 2026-10-08 — The base should be practical, and good at kernels and bootloaders (owner; replaces the morning's principle)

After comparing base CyborgPL with Rust's `core` (operators, `Option` /
`Result`, iterators, formatting without a heap; usable for a kernel as it
is) and asking "Does base Rust have ASP?" (partly: literal range checks and
checked conversions, overflow checked in debug builds only, nothing proven),
the owner: "You know what? Base CyborgPL should be practical, while being
good at kernel & bootloader". This **replaces** the principle of a few hours
earlier ("the base shouldn't be practical to use, even in low-level
environments", below): **the base is meant to be usable on its own, with no
heap and no operating system**, the way Rust's `core` is. What moves into
it is asked one thing at a time. **The operators move into the base** (owner: "`a`"): every
one (`plus`, `minus`, `times`, `pow`, `div.X`, `mod.X`, `equal_to`,
`not_equal`, `<`, `>`, `at_most`, `at_least`, `and`, `or`, `not`, the bitwise
ones), with `.wrap` and `.ieee`, over (b) staying stdkit's; this undoes
2026-10-06's move of `plus` and `div` to std. A type may still define its
own (`op plus`) by the existing power (design claude's reading). Not built. **Numbers in the base mean their true values: ASP's number rules move
into the base** (owner: "`a`, but didn't you just argue that they don't want
this?"): `200` is two hundred, maths exact in between and checked where it
lands (proven, or refused in a final build), wrapping written `.wrap`; over
(b) the machine's maths in the base, ASP on top. Design claude answered the
owner's question: it had argued that kernels wrap on purpose, think in bits
and meet friction; the costs stand, but with the base now meant to be
practical the default decides what happens when nobody thinks, and the
friction has outs (`.wrap`; `0xFF` in a `uint8` is just 255; a reinterpretation
`to.int8.bits[…]`; `no_check` for the unprovable), while (b)'s silent
wraparound is where kernel overflow bugs begin; a kernel core or Limb could
relax it later. **ASP folds into the base entirely** (owner: "`a`"): the
`to.` conversions, checked and raw, `check`, and range types and length
bounds are base CyborgPL, and the core ASP is retired (its ID
`4882d1f3…676f` no longer names a core), over (b) ASP keeping only range
types and (c) ASP keeping all three. stdkit's "needs ASP" goes with it
(design claude's reading). Asked "Would `check` work in kernel & bootloader": yes, it
leaves no code (a proof at compile time, or a compile error). **A run-time
check that fails where there is no operating system freezes the CPU there**
(owner: "yea, freeze there"), with a trap instruction (`brk` on ARM, `ud2`
on x86) so a debugger shows the spot; only working builds have run-time
checks. The owner: "There is another machine with an OS to develop the
bootloader, right?" (yes: it is built on one, then run in an emulator such
as QEMU with a debugger attached, or on the board through a hardware debug
probe). Failures (`-> T or fail[…]`, `otherwise`) are already the base's,
Rust's `Option` / `Result`. **The base gets iterators** (owner: "Let's do
iterators"): sequences described as a chain of steps, lazy and needing no heap,
as Rust's `core` has, over (a) `for` only, which design claude recommended
(`for` covers walking arrays and buffers, and chains of closures are harder
for Precog); asked "What is it for?": shorter common patterns, no lists in
between, handing over a sequence without building it, endless or generated
sequences. Not designed yet. **Genny folds into the base** (owner: "`a`. After finished
compiling, it will report size with a breakdown"): generics and interfaces
are base CyborgPL, so iterators can be ordinary code (an iterator interface,
adapters as generic functions), and the core Genny is retired; over (b)
iterators built into the language with Genny kept a core. The owner had
asked "I heard you said generics would be bad in storage critical stuff"
(`cores.md`: Genny's cost is "paid in program size and build time"; design
claude: one copy per type a generic is used with, the same size as
hand-written for one type, identical copies merged by LLVM, short chains
inlined). **Every build ends with a report of the program's size, broken
down**; design claude's reading of the breakdown, not asked: by Limb, by
function, and the copies each generic produced. **An iterator's `next` returns an `Option`-like value** (owner: "`b`",
after "What's the diff?"), so the base gets a type for "an item, or none",
which can be stored and passed on, as Rust's `Option`; over (a) `next`
failing with `done`, which design claude recommended (one way to say
"nothing", the failures). **Its name is `optional`, its cases `value` and
`none`** (owner: "yea"): `enum optional <T: any> { value: <T>  none }`,
built `optional.value[5]` / `optional.none` and matched bare,
`value 'x' { … }` / `none { … }`, as every enum (2026-09-20). The names
went `Maybe` → "Is `maybe` correct?" → `optional`, lowercase like `list` →
`some` ("What is `some` for?", "So, one value, or multiple?") →
`singular` → `just` ("It could read as 5 values. Wait, where's the word
`value`?") → `value`. Design claude first showed `value` bare (`value[5]`,
`none`), then the prefixed version without saying it had changed; the
owner caught it ("that was not the same as you showed me"), asked "Is the
not bare version correct?" (yes: the 2026-09-20 rule; bare refused, the
generic enum run in the oracle) and "Why is it bare inside `match`?" (the
matched value names the type; the arms copy `otherwise`'s). `value` is also
the label in `list.fill[value 0, count 10]`; the prefix keeps them apart.
**An interface may list a function, so the iterator is one** (owner: "`a`",
after "what" to a first try that asked two things at once): `for` takes any
type whose interface says it has a `next` returning `optional[…]`, the rule
written in CyborgPL like `Ordered`; over (b) the base knowing the shape
without an interface. Design claude recommended (a). Correction: design
claude told the owner an interface could list only `op`s and types, having
looked at the examples only; `design/interfaces.md` has had `fn` entries
from the start (`Printable`), and the oracle runs them. So what was chosen
is the iterator being an interface; nothing new was needed for it. **Each type says once
what it hands out, and `Iterator[int32]` checks it** (owner: "yea"): a
type that fits `Iterator` declares its item type (a type belonging to
another type, Rust's associated type; new to CyborgPL, spelling not
designed), read back as `Iterator.item[<I>]`; `<I: Iterator>` takes any
iterator, `<I: Iterator[int32]>` only those whose declared item is `int32`
(Rust's `Iterator<Item = i32>`); a type is an iterator of one thing only,
so `for` always knows the item. Design claude first offered (a) the item a
second blank filled at the use, recommended; (b) the type declaring it.
The owner asked "diff?", "perf diff?" (none: both resolved while
compiling, one copy per type), "What's `Item`?", "Is `b` a generic?" (no),
"what about usefulness?" — design claude changed to (b), having not said
earlier that (a) lets a type be two kinds of iterator, leaving `for`
unsure, and makes every wrapper carry the item blank — then "what if
allowed both?": (b) with (a)'s brackets as a check, recommended. Not yet
asked: the spelling of a type's item declaration; how adapters chain.

### 2026-10-08 — (dropped the same day) A Turing-complete compile-time sub-language, in a core of its own, behind a `compile_time` power (owner: b; not designed)

The owner: "Feeling like making a turing complete compile time language".
Design claude: CyborgPL already computes anything with known values while
compiling (Precog runs it, 2026-09-22), but compile-time code has no
effects (a const's stdkit call is "an effect, not a value"); asked what it
would help with: deriving code from types (printing, equality, saving),
looking at types, reading files while building, little languages checked
while compiling; tables, generics and format checking are covered
already; its costs are reproducible builds, trusting a Limb's build-time
code, builds that never end (a step budget) and code nobody wrote. Asked
where it should be: not the base (not meant to be practical), not a Limb
(it adds syntax), so a core; inside it (a) CyborgPL itself run at compile
time, as Zig's `comptime`, recommended, or (b) a distinct sub-language, as
C++'s templates are. The owner: "Isn't it `compile_time.io`? If it supports
other narrowings. `b`". So **a distinct compile-time sub-language, in a
core of its own**, run after the program is typed and before MIR (design
claude's placement, so generated code is checked and proven as written
code is), **behind a power `compile_time`, narrowed with `.`
(`compile_time.io`)** as `raw_memory.read` narrows `raw_memory`, so a
program may deny it. **The narrowings** (design claude's set, the owner:
"yea, but there should be a way to be more specific", then, shown the
specific forms and asked whether compile-time code may write files, "`b`"):

```
compile_time.io.read["assets/logo.png", "schema/"]   # only these files or folders
compile_time.io.write["include/"]                    # write, narrowed the same way
compile_time.io.env["VERSION", "BUILD_ID"]           # only these environment variables
compile_time.reflect["Point", "Order"]               # look only at these types
compile_time.emit.fn / .struct / .op                 # generate only these kinds
```

with `deny`'s `.except[…]` and `.for[…]` as for every power; plain
`compile_time` is all of it, and pure computation needs no power (Precog
does it already). Design claude recommended (a) no writing; the owner chose
(b), writing too, for example a header for C. Design claude's reading, not
asked: every path, read or written, is inside the Limb's own folder. **The
sub-language is named Xerpia** (owner: "Xerpia is the sub lang's name. Let's
do the syntax"). **Xerpia is written `inline.Xerpia { … }`** (owner: "`a`"), joining
`inline.ASM`, `inline.LL`, `inline.MSL`, where an item or a value goes,
over a keyword `xerpia { … }` or separate files only (which may come later
for larger code, design claude's reading). On Xerpia's look inside, the
owner: "Nah, I'll just show you" (design claude had offered CyborgPL's look,
a look of its own, or CyborgPL's structure with `$` names). **Xerpia's
memory is manual** (owner: "Manual"), over CyborgPL's ownership, a
collector, one arena per run (recommended) or values only. **On a memory
mistake, Xerpia is lenient by default, and `#bad_girl` makes it an error**
(owner: "Xerpia is submissive by default. With `#bad_girl`, she'll error."),
asked whether a use after free, a double free or a leak should stop the
build (design claude recommended errors, a leak a warning). **The default
carries on silently, as C does** (owner: "`b`"): a use after free reads what
was last there, a double free is ignored, a leak is freed when the run
ends, with no warning; over (a) the same with a warning naming the line,
which design claude recommended. **No Xerpia files: Xerpia lives only in
`inline.Xerpia { … }` blocks, and `#bad_girl` goes at the top of a block,
covering it** (owner: "I mean, I don't think C++ templates has a separate
file. Does it?", then "yea"); this drops design claude's earlier reading that
separate files might come later. **The core is named Xerpia too**, and **a
variable is declared `bin16 %^x = 1000;`** (owner: "Xerpia is the core's name
too. var declare syntax: `bin16 %^x = 1000;` And whatever the fuck you have
to allocate for that. `%` means local, `^` means level, one `^` means
variable, and then stacking could result in another thing (const, fn,
whatever)."): the type first, then the name with its marks; `%` marks a
local (as in LLVM IR), and the count of `^` says what kind of name it is,
one `^` a variable, more for other kinds. **The ladder: `^` a variable,
`^^` a constant, `^^^` a function** (owner: "yea. People are gonna have a
stroke reading this shit, especially with manual. But, whatever"), more `^`
meaning more fixed, room left above (a type at `^^^^`, design claude's
example, not asked); `fn %^^^twice [bin16 %^n] -> bin16 { … }` was design
claude's illustration of where the marks go, not a ruling on functions.
**No `fn` in front: `^^^` alone makes a function**, and **`@` marks a
global**, the twin of `%` (owner: "Btw, you don't say `fn` infront. `@` means
global."). Not yet asked: how wide a global is (one block, or every Xerpia
block in the Limb); how a function's parameters and result are written;
what
allocating "for that" means under manual memory; the step budget. The fourth IR LNIR (2026-10-01)
is unrelated: it is for loops.

### 2026-10-08 — No split after all: stdkit stays one Limb; kernels get a core or Limb of their own later; the base "shouldn't be practical to use" (owner; the principle replaced the same day, above)

Right after choosing (c) below, asked whether the heap-free part should be a
Limb or a core ("I thought it was a core") and whether "a kernel really
want[s] ASP" (design claude: not necessarily; kernels wrap on purpose and
think in bits, and the owner's own "Signatar + IEG is for kernels", 2026-10-06,
has no ASP; so the heap-free part should need only IEG, its operators
following whatever number rules the program has), the owner: "Or just keep
the base the same, but we'll later add a specialized kernel core/limb".
Asked whether that replaces (c): "Yeah. the base shouldn't be practical to
use, even in low-level environments, but that doesn't mean the base should
have nothing either." So **(c) is undone: stdkit stays one Limb, operators
and heap together; the base is unchanged; a kernel core or Limb comes
later**, designed for kernels (their maths, their testing), not designed
now. What `plus` does without ASP waits for it. **The owner's principle
for the base: it is not meant to be practical to use, even for low-level
work, yet it is not empty.**

### 2026-10-08 — (undone the same day) stdkit splits: a heap-free Limb holds the operators, stdkit the rest (owner: c; not built)

The owner: "Should we make the base less lower-level?" Design claude: not
the base (low on purpose, 2026-10-06), but one gap: the operators are
stdkit's, and stdkit brings the heap (lists, maps, text, printing), so a
kernel cannot add two numbers without an allocator it cannot use; Rust
splits `core` / `alloc` / `std` for this. Offered (a) leave it, (b) the
operators back into the base, undoing 2026-10-06, (c) split stdkit into a
heap-free part and the rest, recommended. Asked what (c) does, then "Do
kernels need ASP?" (not strictly: ASP costs nothing at run time in a final
build, every check proven or refused; kernels are where silent overflow
becomes a security hole; the very bottom, a bootloader or register
bit-twiddling, may skip it). The owner: "`c`, if it's correct". Design
claude checked: every operator works without a heap (arithmetic,
comparisons, logic, bitwise, `.wrap` / `.ieee`, WSMW's fixed-size wide
integers, `equal_to` on text literals); `++` makes new text, so it stays in
stdkit. So **a heap-free Limb holds every operator, negation,
`round` / `floor` / `ceil` / `trunc` and the everyday interfaces (`Number`,
`Ordered`)**, and **stdkit keeps lists, maps, text operations and `++`,
printing, `format`, input, time and `exit`**, including the heap-free part.
The base is unchanged. Design claude's readings, not asked: the heap-free
part needs ASP and IEG, as stdkit does. Not yet asked: its name; how a
program listing only stdkit reaches the operators (stdkit passing them on,
or listing both).

### 2026-10-08 — Shadowing is `shadow 'x' := 2;`, a verb in `let`'s place; `$:=` and `$=` are gone (owner: yea)

```
let 'x' := 1;
if true {
    shadow 'x' := 2;            # a new 'x' for this block
}
shadow 'x': int32 = 3;          # typed
shadow mut 'y' := 0;
let no_shadow 'total' := 0;     # its twin, unchanged
```

Asked how shadowing looks today (`let 'x' $:= 2;`, `let 'x': int32 $= 3;`
since 2026-09-14; a plain `let` over a visible name refused; `no_shadow`
forbidding it), the owner: "I think we should change it to `shadow`".
Design claude showed `let shadow 'x' := 2;`, in `no_shadow`'s place; the
owner: "Not `let`"; design claude showed `shadow` taking `let`'s place, a
verb like `change` (`let` declares, `shadow` declares again over an old
name, `change` changes); the owner: "yea". So `$:=` and `$=` are old
spellings, refused naming `shadow`, and `$` is no longer part of the
language. `shadow` is a keyword. Built in the oracle; reject tests
`old_shadow_walrus`, `old_shadow_eq`, `shadow_with_let`. The owner also
asked about abbreviations for `grapheme`: there are none (a unit is
written out, 2026-09-22). Shown `->` beside `>>` (with `<<`, `<<>>`) for the arrows,
the owner kept `->`: "Yeah, nvm. Even though `>>` is faster to write, my eyes
are getting confused with it."

### 2026-10-08 — `pos` on every position, `list.remove` and `str.at` / `str.slice` too (owner: a)

```
list.remove[lend.mut 'xs', pos 1];
let 'third' := str.at.grapheme[lend 's', pos 3] otherwise `?`;
let 'start' := str.slice.grapheme[lend 's', pos 1, pos 4] otherwise "";
```

Compiler claude, building the revision, read `list.remove`'s position as
bare, as design claude had built it: `pos` had gone only where `#strict`
put it (an index, `list.slice`, `simd.load` / `store`). The owner asked
what that meant ("Wdym?"); design claude showed the bare positions left
(`list.remove`, `str.at.<unit>`, `str.slice.<unit>`, so `list.slice` said
`pos` and `str.slice.grapheme` did not) and offered (a) `pos` on every
position, recommended, or (b) leave them. The owner: "Let me see `a`",
then "`a`, if it's correct". Design claude checked they are positions
(`design/lists.md`: "by 1-based position"; `strings.md`: "the 3rd",
"graphemes 1 to 5"; `map.remove`'s key is not one), and corrected its own
example, which had left out the `otherwise` that `str.at` and `str.slice`
need. Built in the oracle; reject tests `remove_needs_pos`,
`str_at_needs_pos`.

### 2026-10-08 — In `Synthia.ttsr`, a Limb's powers are `power.` lines in `[my needed things]` (owner: a; relayed by synthia claude)

```
[my needed things]
power.operators_on_builtin_types = yes
power.raw_memory.read = yes
power.c_calls.functions = sqrt, sin
```

Synthia claude offered (a) `power.` lines in `[my needed things]`, beside
`core.` and `limb.`, recommended, or (b) a `[my powers]` section. The
owner asked "Does this say what it needs, or what it allows the others";
synthia claude answered that it is what the Limb itself uses, an honest
label, allowing or forbidding others being `deny`. The owner: "`a`". The
last line is written to `CCA.cplr` as
`cca::Power.c_calls.functions["sqrt", "sin"]`. Built in Synthia
("decision.manifest.powers"): only the shape is checked, since cores may
add powers; `= no` is refused. `examples/limbs/areas` written as a
`Synthia.ttsr` syncs to its `CCA.cplr` byte for byte except `default =
true`. Synthia also
caught up with the 2026-10-07 revision (24df7d3). Separately, the key in
`limb.<key>.id` is the manifest writer's own label (owner: "Idk, because
doesn't the user of Synthia name that?"), so std's rename to `stdkit`
needs no ruling there.

**Then: a program takes powers away in a `[my denied powers]` section**
(owner: "`a`", relayed by synthia claude), over (b) `deny.` lines in
`[my needed things]`; synthia claude recommended (a), a denial being a rule
set for others, not a need:

```
[my denied powers]
raw_memory = yes                 # all of it
c_calls.functions = system       # the things named
c_calls.except = sqrt, sin       # all but these
inline_assembly.for = fastjson   # only to that Limb, by prefix
```

written to `CCA.cplr`'s `deny`; `= no` refused. Built in Synthia (bd3dd18).
Design claude ran the oracle on Synthia's written `CCA.cplr` for
`examples/limbs/areas`: the multi-line `deny` with `.except["sqrt"]` parses;
with a `MAIN` added it runs, and a function using `inline.ASM` (its powers
listed) is refused naming the `deny`.

### 2026-10-07 — The plugin interface: compiler claude's proposal, answered question by question (owner; not built)

`cca/PLUGINS.md` (735fc8c0), written by compiler claude at the owner's
request ("yea", design claude asking it for a proposal only). Three
boundaries: base ↔ codextra (small, stable: the codextra returns IR, a
list of power uses and messages, and the base checks the uses itself),
codextra ↔ core (Signatar's to define), base ↔ core (loading, identity,
dependencies, the power catalogue). Its six questions, as the owner
answers them:

- **Q1, how a core adds syntax: a fixed menu of hooks** (a statement
  word, a prefix head like `to.`, a narrowing after an operator, a word
  operator with its level, a prefix operator, a type head, a modifier on
  `for`, a marker before a statement, an item kind, an `inline.` language,
  a spelling of an existing token, the characters a name may hold), **with
  raw token runs allowed inside the statement-word and prefix-head hooks
  only**; two cores claiming one entry are refused when the manifest is
  read (the owner: "yea", to compiler claude's recommendation, which
  design claude seconded), over token rewriting everywhere (Rust-macro
  style) and grammar fragments merged into Signatar's parser.
- **Q2, whether a core's words take over names: reserved only in the
  limbs that list the core** ("`concur` is MCPS's word"), **and a core may
  instead leave a word unreserved, found by its position** (the owner:
  "`a`, `b` depends on the core"), over every core's words being keywords
  everywhere (as `on` and `my` became).
- **Q3, whether Precog trusts a core's contracts: yes, every core's**
  (owner: "`a`"). Compiler claude had recommended trusting only verified
  (official) cores, unverified ones' operations being opaque; the owner:
  "Holy registry lock in". Design claude offered (d) trust by proof, any
  core whose contracts RRx2 confirms, whoever wrote it; the owner: "Atp,
  they're just gonna fork our entire project instead", then "`a`": every
  core's contracts are trusted, the core being trusted already by the
  warning's terms. So Precog's proofs are as sound as the cores a program
  lists, and no core is privileged by who wrote it.
- **Q4, how the deep official cores move out (ASP's number meaning,
  Genny's generics, IEG's marking rule): real separation** (owner:
  "`b`"): Signatar opens its type checker and Precog to the plugin
  interface far enough that ASP, Genny and IEG are written outside it, so
  anyone could write their own (an "ASP2" whose numbers saturate). Over
  (a) policies kept inside Signatar that only the official cores switch
  on, and (c) (a) now, (b) when someone asks, which compiler claude and
  design claude recommended. The cost, accepted: a far larger interface,
  every change to Signatar's checker being a change to it. In keeping with
  Q3: no core is privileged by who wrote it.
- **Q5, what a plugin file is: the OS's own native library** (`.dylib` on
  macOS, `.so` on Linux, `.dll` on Windows, behind a stable C ABI) (owner:
  "the OS lib thingies file type"), over a WebAssembly component in a
  sandbox (compiler claude's recommendation for third-party cores) and a
  separate process. Asked along the way: a core mods the compiler either
  way (the sandbox would only have limited what else it could do on the
  machine while compiling); the owner asked why the language was becoming
  restricted (Rust's procedural macros and `build.rs` run unsandboxed too);
  a dylib is about as fast as code built into CCA, and **the finished
  program is the same either way**: a plugin runs only while compiling and
  hands back IR. A core is trusted when listed, as the approved warning
  says. One build per platform.
- **Q6, whether an unofficial core may run in `--assert-proofs` and in the
  registry's build of a published limb: yes, in both**, the registry
  building inside a throwaway, sealed machine so a core's native code
  cannot harm its servers (design claude's suggestion; the owner: "yea"),
  over refusing them there (compiler claude's recommendation), which
  would have left unofficial cores out of published limbs.

Not asked, left as the proposal has it: the official cores built into CCA
("the common case loads nothing"); with native plugins equal in trust and
speed, being built in would be no privilege. The two gaps stay open: which powers MCPS, the GPU
cores, WSTN and MTM add, and IEG's raw operations. Compiler claude's
build order (split CCA; the power ledger; the hook menu; MTM, WSTN; MCPS
and the GPU cores; ASP, IEG, Genny; external loading) stands as
engineering, with Q4's real separation in step 6.

### 2026-10-07 — The dangerous abilities are powers too: Signatar grants them to every limb, and a program may deny them (owner: a; not built)

```
cca::Manifest[
    …,
    deny = [cca::Power.inline_assembly, cca::Power.c_calls,],    # design claude's spellings
]
```

Designing the powers (owner: "design", over asking compiler claude for
the plugin interface first). The four decided so far all let a limb
extend the language (operators on built-in types, prefix operators,
adding conversions, plugging into `for`). Design claude asked whether
the abilities a program may not want its dependencies to have
(`inline.Assembly`, `extern.C` calls, raw pointers) are powers too: (a)
yes, **granted by Signatar itself to every limb by default**, so pure
Signatar's Hello World through C still works, and a program may `deny`
them; (b) no, they stay unrestricted. Asked the difference: under (a)
every build lists which limb uses which, `deny` stops a build that
needs one, and an upgrade that starts using one is flagged (2026-10-06);
under (b) only reading the source tells. The owner: "`a`".

**Three of them, `inline_assembly`, `c_calls` (C functions and
variables), `raw_memory` (pointers as values, `ptr.read`/`write`/`cast`,
addresses as numbers, volatile, memory with no owner)**, design claude's
grouping by what each lets a library do, over one `unsafe` power or a
finer split. The owner: "cores can add abilities to that list. And I
think that's already fine, but there should be a way to be a lot more
specific". So **a core may add powers of its own** (design claude's
reading, asked: alongside Signatar's, which still sets the limits no
core lifts, 2026-10-06), and **a power can be named more specifically**
than these three: **narrowed with `.`** (`raw_memory.read` /
`raw_memory.write`, `c_calls.functions` / `c_calls.variables`) and **exact
things named in `[ ]`** (`cca::Power.c_calls["sqrt", "sin"]`, only those C
functions) (owner: "yea"). **One field, `deny`, with `.except[ … ]` and
`.for[ … ]`** (owner: "yea", to design claude's recommendation, over an
`allow_only` field beside it):

```
deny = [
    cca::Power.c_calls.except["sqrt", "sin"],    # no C calls but these two
    cca::Power.inline_assembly.for["fastjson"],  # only this limb may not run assembly
],
```

so a program still only takes powers away (2026-10-06). **The narrowings,
ruled 2026-10-07** (owner: "`a`"; they had been design claude's sketches,
the owner having ruled the dot and the brackets: "Didn't I already decide
this?"): `inline_assembly` has none; `c_calls.functions["name"]` and
`c_calls.variables["name"]`; `raw_memory.read` (`ptr.read`, `ptr.at`,
reading by address, volatile reads), `raw_memory.write` (`ptr.write`,
writing by address, volatile writes), `raw_memory.cast` (`ptr.cast`,
bytes read as another type).
**`.for[ … ]` names a limb by its prefix** (owner: "`b`"), the word code
writes before `::`, unique within a program and what the code already
shows (`fastjson::parse[…]`), over its name (names repeat) and its
registry ID (unreadable, and a local limb has none); a prefix Synthia
renamed is the renamed one.
**`operators_on_builtin_types` is needed only by an operator whose every
operand is a built-in type** (`op mean ['a': bin64, 'b': bin64]`); one
taking any of the limb's own types (`op scale ['v': Vec2, 'k': bin64]`)
is free, since it cannot change what `bin64 plus bin64` means in anyone
else's code (owner: "`a`"), over any built-in operand needing it.
**Built in the oracle the same day** (compiler claude, having split CCA
into `cca-base`, `signatar` and `cores/*`, asked for the oracle first, as
the spec): `powers` and `deny` read, every use found from the parse, and
checked granted, listed, not denied; examples and tests gained their
`powers` lines.
**A limb lists every power it uses, the dangerous ones included**
(owner: "`a`"): `powers = [cca::Power.c_calls,]` in its manifest, and code
using an unlisted one is refused ("`fastjson` calls C but does not list
`c_calls`"), over the compiler working it out and only reporting it. The
manifest is an honest label the registry can show before a download, and
an upgrade that starts using a new power is found by comparing lists.
**The program's own Limb lists its powers too** (owner: "`a`"), the same
rule as any limb, so a program can later be published as one unchanged;
a lone `.cpls` with no `CCA.cplr` has every power, as it has every core.
**Vouching is a power IEG grants** (owner: "`a`"): `cca::Power.vouch` for
`!check` and `cca::Power.vouch.c` for `NO_FUCKING_CHECKS` (design claude's
spellings), listed by a limb that vouches and deniable like any power
(`deny = [cca::Power.vouch.for["fastjson"],]`), over leaving it to the
build's list of `!check` sites. The owner: "And yes, if you disable
`check`, it's just not gonna work": a limb that vouches does not build
where its vouching is denied; that is the point.

### 2026-10-07 — A shift's width comes from the shifted number, never from its count; `shl.masked` / `shr.masked` shift as the chip does (owner)

```
let 'c': int8 = 9;
let 'r': int64 = ((0 minus 53) shr 'c') mod.floor 23;    # a 64-bit shift: 22
```

Check claude's question (corpus 7e9b1361): a bare shifted number took the
count's `int8` in CCA (refused: 9 past 8 bits) and the destination's
`int64` in the oracle (22); 2026-09-23 (a), "any operand whose type is
known", did not rank them. Asked how the hardware does it: ARM's `lsl x0,
x1, x2` shifts at the shifted register's width, the count's size beside
the point (taken modulo 64 or 32); x86's `shl rax, cl` always takes an
8-bit count; C and Rust give the result the left side's type. The owner:
"yea": **the width is the shifted number's own type, or, when it is
bare, its context; the count never decides it.** The oracle already did
this; CCA changes. **The hardware way is `shl.masked` / `shr.masked`**: the
count taken modulo the width (70 on 64 bits shifts by 6) and the bits
pushed past the width cut off, with no check site; plain `shl` / `shr`
stay checked. Design claude offered `.wrap` (the family's word) and asked
whether it should do both of the chip's behaviours; the owner: "both. But
is `wrap` the right word? Not consistency, but the meaning". Design claude:
the count and the number do wrap around their ranges, but bits pushed off
do not come back at the other end, so on a shift "wrap" reads as rotating
(ARM's `ror`); offered `.hardware` (the owner: "sounds a bit like it
depends on where it's running": it does not, CyborgPL defines it), then,
by meaning alone, `.masked` (cut to fit: the count masked to the width,
the answer to it) against `.wrap` (Rust's `wrapping_shl` masks the count
too). The owner: "Whatever's accurate, man": `.masked`. A shift with no
width anywhere is refused for `shr` as for `shl` (design claude's,
matching CCA). `examples/shift_masked.cpls`, reject test `shr_no_width`.
**The shifted number must fit its width first**, as a bitwise operand
must (check claude's question; the owner: "`a`"): `(0 minus 1) shr 0` at
`uint32` is refused ("-1 does not fit uint32: a shifted number is taken
at its width (`shr.masked` takes its bits instead)"), over shifting the
exact number and checking only the answer. `.masked` takes it as its
bits: `(0 minus 1) shr.masked 1` at `uint32` is 2147483647 (the oracle had
shifted the negative number first, fixed the same hour). Reject test
`shift_operand_too_wide`.

### 2026-10-07 — A bitwise operand must fit the operator's width; `and.bit.wrap`, `or.bit.wrap`, `xor.wrap` take its bits (owner: yea)

```
'v' and.bit (1 minus 7)          # refused: -6 does not fit uint32
'v' and.bit.wrap (0 minus 1)     # -1's bits are all ones: 'v' unchanged
```

Check claude's question (seeds in 7e9b1361): CCA (c1973f02) refused an
exact operand that does not fit, the oracle combined the bits and checked
only the answer. Design claude first recommended the strict way, then,
asked "Is `a` consistent?", checked: it matches conversions
(`to.uint32[-6]` is refused) but not the `.wrap` family, which takes an
out-of-range operand as its bits (`'v' plus.wrap (1 minus 7)` is
4294967295), so recommended the oracle's way instead. Asked the
difference and the cost (strict: a compare per unproven operand; bits:
none, only the answer checked), the owner: "Is there like a way to say how
in the dot chain". Design claude: `.wrap`, which already means "at the
type's width, as bits"; the owner: "yea". So the **default is strict**
(an operand must fit, a check site) and **`.wrap` is the hardware way**:
operands taken as their bits, the answer an N-bit pattern that always
fits. Shifts and `not.bit` unchanged. `examples/bitwise_wrap.cpls`,
reject test `bitwise_operand_too_wide`.

### 2026-10-07 — `!` gives way to words: `immut`, `no_check`, `no_cheat`, `not`, `not_equal`, `not_std`; std will be renamed (owner; not built)

The owner: "I'm feeling like replacing `!` with characters". Design
claude listed `!`'s jobs (`!mut`, `!check`, `!cheat`, `!$`, the spaced
`! 'x'`, `!==`, `!std.int24`) and asked whether that meant words, another
symbol, or something else, and for which. The owner: "`immut`,
`no_check`, `no_cheat`, `<correct grammar> shadow or whatever`, `not`,
`not_equal`, `not_std`. and the std limb's name could change, so it
won't be confused". So the main spellings become **`immut`** (for
`!mut`), **`no_check`** (`!check`), **`no_cheat`** (`!cheat`), a
**shadow word** still to be chosen (`!$`), **`not`** (the spaced `!`),
**`not_equal`** (`!==`) and **`not_std`** (`!std.int24` becomes
`not_std.int24`); this reverses 2026-10-06's "`!` is the main spelling
of not". **std's name will change** so `not_std` is not confused with
it; the new name is not chosen. Not asked yet: the shadow word; whether
`==` becomes `equal`, the twin of `not_equal`; whether the `!` spellings
move to MTM or are refused; what MTM's `nocheck` becomes;
`NO_FUCKING_CHECKS` is untouched.

**The shadow word is `no_shadow`** (owner: "If `a`, `no_check` has to
change. And I think it becomes too long. So, `c`"), over (a)
`no_shadowing`, which design claude recommended for its grammar, and (b)
`unshadowable`: `let no_shadow 'limit' := 10;`.

**Comparisons become words, but `==` is `equal_to`, and `<` and `>`
stay** (owner: "`b`, with `equal_to`, `>`, `<` (because, making `equal`
could blend it with the reader's `=`, when they pronounce it in their
head, or out loud)"). Design claude had offered (a) only `==` to
`equal`, (b) every comparison a word (`equal`, `not_equal`, `less`,
`greater`, `less_or_equal`, `greater_or_equal`), recommended, (c) `==`
kept. So `'a' equal_to 'b'`, `'a' < 'b'`, `'a' > 'b'`.

**`at_most` and `at_least` replace `</==` and `>/==`; `!==` is
`not_equal`** (owner: "If `c` is correct, then yes. But `not_equal_to` is
a bit longer, but it has more eye movement to see two variables there.
So, it would be `not_equal`"). Design claude first offered `<=` / `>=`,
saying equality being a word removed 2026-09-29's reason; the owner:
"`>=` greater than assign?", and design claude agreed it was wrong (the
reason, no lone `=` inside a comparison, still holds). Offered (a)
`less_or_equal_to` / `greater_or_equal_to`, (b) `</==` / `>/==` kept
(built from an `==` the language no longer has), (c) `at_most` /
`at_least`, recommended; and `not_equal_to` to match `equal_to`, which
the owner declined. The family: **`equal_to`, `not_equal`, `<`, `>`,
`at_most`, `at_least`** (`'i' at_most 'n'` is `'i'` ≤ `'n'`).

**MTM is gone** (owner: "MTM is gone."), asked whether the replaced
spellings (`!mut`, `!check`, `!cheat`, `!$`, `! 'x'`, `==`, `!==`, `</==`,
`>/==`, `!std.`) move to MTM or are refused, design claude recommending
refused. With no MTM, **every other spelling is refused**, naming the
main one (design claude's reading): the `!` forms, the old comparisons,
and everything MTM held (the fourteen other arrows, `&` / `&mut` / `&@`,
Rust's number names, `bf16` / `bfloat16`, `inline.Assembly` / `inline.S` /
`inline.LLVM…` / `inline.Metal_Shading_Language`, `nocheck`, the word
`not` as MTM's, `!std.i24`). MTM's registry ID
`9c6c7730…3a56` is retired.

**std is renamed `stdkit`** (owner: "`stdkit`"), over design claude's
`parts`, `kit` and `common`, so it is not confused with `not_std`:
`stdkit::print.useful.stdout[…]`, still `bare.optional` for everything.
Design claude's readings, not asked: its registry ID and version stay
(an ID names the Limb, not its name: `fff4302d…1936`); the prefix
`stdkit` is reserved as `std` was; every rule that named std (needs ASP
and IEG, refused unlisted) now names `stdkit`; `not_std.int24` keeps its
`std` meaning "standard width", unrelated to the Limb.

### 2026-10-07 — Syntax revision, loops: every `range` is labelled, `range[from 1, to 4, by 1]`, `by` always written (owner: a; not built)

```
for 'i' in range[from 1, to 4, by 1] { … }
for 'i' in range[from 10, to 0, by -1] { … }
```

The owner: "Feeling like a syntax revision right now, likely not gonna be
removing, but adding for clarity", then "Let's do the loops, how do you
write a loop using `#strict`?" (shown side by side: `#strict` labels the
range, writes `temp`/`perm` on every `for`, `pos` on every index, `mut`
or `!mut` on every declaration; `while` and `loop` are the same), then
"question me". Asked whether the labelled range becomes the normal
syntax: (a) labels and `by` always written, as `#strict` does; (b) labels
always, `by` only when not 1, a count-down without `by` refused,
recommended; (c) unchanged. The owner: "`a`". So **every range is
`range[from …, to …, by …]`**, everywhere, and the unlabelled form is
refused naming the labelled one (design claude's reading, one spelling).

**`temp` stays the default and may be written** (owner: "`b`, but the
default could be written"), over (a) `temp` or `perm` on every `for`, as
`#strict` does; design claude had recommended (b). Writing `temp` was
already allowed outside `#strict` (checked in the oracle), so this keeps
today's rule.

**`pos` comes back on every index** (owner: "`a`"): `'xs'[pos 'i']`,
`'xs'[pos 2]`, over (b) staying removed as since 2026-09-27 ("Remove
`pos`"), which design claude recommended. Design claude's reading, not
asked: everywhere `#strict` wrote `pos`, so every position a list or
array call takes too (`list.slice[lend 'xs', pos 2, pos 3]`), and the bare
form refused naming `pos`.

**`count` is required in `list.fill` everywhere** (owner: "`a`"),
`list.fill[0, count 10]`, the twin of `pos`, over (b) keeping it
strict-only, which the owner pointed out cannot be: "“count stays
strict-only”, it can't" (`#strict` is being deprecated). The owner then
asked "What does `0` there mean?" (the value every element starts as).
Asked whether the value is labelled too: (a) no; (b) `value`,
recommended; (c) `each`. The owner: "`b`". So **`list.fill[value 0, count
10]`**.
Its twin, asked: `simd.fill[value 0]` too, (a) required, recommended so
both `fill`s read alike, or (b) `simd.fill[0]` unchanged, its one
argument unambiguous. The owner: "`a`". So **`simd.fill[value 0]`**.

**`mut` only where a variable changes, as today** (owner: "`b`"), over
(a) `mut` or `!mut` on every declaration, as `#strict` does, and (c) (b)
with `!mut` allowed for stress, which design claude recommended. So no
word means it never changes. Design claude's mistake, found after: (b)
and (c) were the same, since `!mut` may already be written outside
`#strict` (`let !mut 'x' := 1;` runs in the oracle; parameters and fields
take it too, `examples/mutability.cpls`), so today's rule stands
unchanged. The owner then: "I'm feeling like replacing `!` with
characters".

### 2026-10-07 — A slimmer base: `brain16`, non-standard integers, Unicode text and the case marks move out; `#strict` will be deprecated (owner; not built)

The owner: "Let's make base CyborgPL slimmer". Design claude listed what
a kernel needs at the bottom (integers, `bool`, structs, enums,
functions, control flow, borrows and Precog, raw memory, `inline.ASM`,
`on.target`, fixed arrays) and candidates to move into cores: floats,
decimals, Unicode text, SIMD, wide integers, ranges, the case marks,
`test fn` and `#strict`, first recommending floats and decimals out
because Linux keeps floating point out of kernel code. The owner: "Wait,
isn't `bin` one of the fastest types? Why is it banned in Linux?"
Design claude: it is fast; Linux does not save the floating-point and
SIMD registers on entering the kernel, so kernel code using them wraps
itself in `kernel_fpu_begin()` / `kernel_fpu_end()`; "banned" was too
strong, and it is a rule for kernels, not a reason to take floats out.
The owner: "Won't we be adding hardware `deci` support?" (yes, on IBM
POWER and z, if-and-when, 2026-09-29's layout ruling; software elsewhere),
"Currently, I'm sure that we'll be removing `brain`, non standard types,
spelling ways, and unicode out of base." Asked what "spelling ways"
meant: "I meant `#strict`, and `case.sens...`. Also, we'll be
deprecating `#strict`." So **out of the base, into cores: `brain16`;
the non-standard integers (`!std.int24`, `!std.uint256`, …); Unicode text
(`str.utf16`, `str.utf32`, `char.point`, `char.grapheme` and the Unicode
tables; the base keeps byte text and `char.byte`); `case.insensitive` /
`case.sensitive`; and `#strict`**, which **will be deprecated**. **Then: `#strict` is
removed, not moved to a core** (owner: "We ain't no C stability, remove it
now. But not literally now"), after design claude pointed out "moved to a
core" and "deprecated" contradicted each other: it is refused once the
revision is built, over a slow deprecation or a core. Its useful parts
became normal syntax (labelled ranges, `pos`, `count`); its five
leftovers (parameter types, return types, `gives`, `@x` on a returned
borrow, the working build refusing unproven sites) are each ruled on
separately. **Parameter and return types are required on every function**
(owner: "`a`"), asked (a) every function, (b) `pub` functions only,
first recommended, (c) optional as today; asked what `fn area ['r'] {
return 'r' times 'r'; }` even meant (a parameter whose type Precog
infers from every caller, 2026-09-19, and a return type from the
`return`), design claude then recommended (a), a function that works for
several types naming them with Genny's `<T>`. So 2026-09-19's untyped
parameter goes: `fn area ['r': bin64] -> bin64 { … }`. **The arrows are
short again: `->`, `<-`, `<->`** (owner: "Short, the neck & eye movement
is less"), shown the same functions, a failing one, a lent return, a
closure and assembly bindings with `->` and with `-->`; this reverses
2026-10-06's "Signatar supports long, not short anymore". Design claude's
reading: the whole family goes short (`<-` for a binding out, `<->` both
ways), and with MTM gone the long ones are refused naming the short. **A case's value says `is`** (owner: "`is`"):
`match 'l' { empty { is "free"; } claimed { is "taken"; } }`, and in
`otherwise` cases too, replacing `#strict`'s `giveS`. Asked whether
`#strict`'s `gives` should be required everywhere, the owner: "Does
`gives` make sense?" Design claude: not really, it collides with `give`
(the reason for 2026-09-29's capital S); offered (a) `is`, recommended,
(b) the arrow, `empty -> "free"`, (c) the bare value; shown `is` in a
simple, a multi-statement, a nested and an `otherwise` case, then beside
`gives` (`gives give 'box'` against `is give 'box'`). Design claude's
reading: required wherever a case gives a value, the bare value refused
naming `is`; a case that only does something has none. **A returned borrow names its lifetime group only when there is
a choice** (owner: "`b`"): with two or more borrowed inputs, `-> lent@x
T` is required; with one, it is inferred, close to Rust's rule; over (a)
always, as `#strict`, and (c) never. The owner then: "And man, we have to
change the fn lifetime and generics syntax". Shown today's (`fn max <T>: Number ['a':
<T>, 'b': <T>] -> <T>`, `fn pick <T>: any | @x [lent@x 'xs': list[<T>]]
-> lent@x <T>`) and offered (a) `<T: Number>` announced then bare `T`,
recommended, (b) a `where` clause, (c) a `type` parameter, the owner:
"`fn max [<T: Number,> ] ['a': <T>, 'b': <T>] -> <T> { … }` `<>` makes it
look like a placeholder, which a generic could be. And that `,` was for
more generics". So **a function's blanks are announced in their own
bracket before the parameters, in one `< >` with commas between**, and
**every mention keeps `<T>`**: `fn pair_up [<A: any, B: Hashable>] ['a':
<A>, 'b': <B>] -> Pair[<A>, <B>]` was recorded first; then, asked where
lifetime groups go ((a) in the same bracket, recommended, (b) their own,
(c) unannounced), the owner: "`fn pick <T: any> [@x, @y] [lent @x 'a':
list[<T>], lent @y 'b': list[<T>]] -> lent @x list[<T>]`?" So **the
signature is `fn name <generics> [lifetimes] [parameters] -> return`**:
blanks in `< >` straight after the name, commas between (no outer `[ ]`),
lifetime groups in a bracket of their own, and **`lent @x` spaced**. The
owner: "yea" to that, **with structs, enums and interfaces the same**
(`struct Pair <A: any, B: any> { … }`), "but what if they want `-><T>` as
an op?" Design claude had suggested the lexer always end `->` at the `>`;
offered (a) keep 2026-09-20's "a run of operator characters is one
operator", the space in `-> <T>` required and a missing one met by an
error saying so, recommended, (b) `->` always its own token and no
operator starting with `->`, `<-` or `<->`, (c) the parser splitting
`-><` only where a return type goes. The owner: "`a`". So `-><` stays
someone's possible operator, and `-> <T>` keeps its space. **The working build keeps checking unproven sites at run time;
only the final build refuses them** (owner: "unchanged, if it's already
good"), over (a) the working build refusing them too, as `#strict` did,
and (c) a build switch; design claude recommended (b). The owner had
asked "How the fuck would you test a runtime-dependant overflow with
`#strict`?": you could not, the program never building until the
overflow was made impossible. That was `#strict`'s last leftover. **`[ ]` keeps all its jobs** (calls, building a struct,
indexing, lists, type arguments, parameters, labelled built-ins): shown
them, with (a) no change, the labels and `pos` having removed the
ambiguity, recommended, or (b) calls in `( )`, the owner: "It's likely
gonna collide either way, because `()` is also grouping". **`format[…]` makes text the way `print.format` prints it**
(owner: "`a`"): `let 'x' := format["Hello, {}! You have {} items\n",
'name', 'n'];`, stdkit's, `bare.optional`. The owner had written `let 'x' :=
format "Hello, World!\n"`, "Just like format print?"; design claude
offered the values in brackets with holes, inline (`{'name'}`), or after
`with`, recommending inline; the owner: "Then it's not the same format".
Offered (a) `format[…]` taking exactly what `print.format` takes, (b) both
moving to inline values, reopening 2026-10-01's numbered holes; design
claude recommended (a), and, asked whether it collides with arrays or
lists, answered no: a word before `[ ]` is a call, a bare `[ ]` a list, a
quoted name before it indexing. One format language, used to print or to
make text. **No `print.weird.format`** (owner: "`b`"): asked "Could you do
something like `stdkit::print.weird.format.useful.stderr`?", design claude
first offered it (without `.useful`, each hole choosing its form); the
owner: "Does format require shit being in one `""`?" (yes: a format is one
literal so its holes are checked; anything else is a vouch site), then
"No, I meant, wouldn't that make it incompat with `weird`?" Design claude
agreed: `.weird` prints every `<-` piece in order, a format's first piece
is a template and the rest fill holes, so it withdrew (a). `.weird` gets
shapes by sending `format[…]`'s text down an arrow: `print.weird.useful.
stdout <- "hex: " <- format["{:x}", 'n'] <- \n;`. **A print goes to stdout unless it says `.stderr`** (owner:
"default to out"): `print.useful["Hello, World!", \n];`, over (a) the
destination always written and (c) `print` defaulting the form as well;
design claude recommended (b). Asked "Is `b` consistent? I mean, the ops
don't default `ieee`", design claude: they do default, plain `plus` being
the common case and `.ieee` the written variant; the language defaults
where one choice is ordinary (`plus`, `temp`, `case.sensitive`) and
requires a choice where both are reasonable (`div.X`, the `bare` marks,
`print.useful` / `print.stored`). Design claude's readings, not asked:
every print (`.format`, `.weird`, `.auto_newline`) takes the default, and
`.stdout` may still be written, as `temp` may. **`lend` / `lent` stay** (owner: "`a`"), the caller lending and
the parameter lent, over (b) `lend` on both sides; design claude
recommended (a). **`print.useful.auto_newline` becomes
`print.useful.auto_newln`** (owner: "`auto_newline` should be
`auto_newln`"): `print.useful.auto_newln["Hello, World!"];`. **`:=` and `=` stay** (owner: "`a`, it's fucking iconic, bro"):
`let 'x' := 1;` with the type inferred, `let 'x': int32 = 1;` with it
written, over (b) always `=` and (c) always `:=`. **`and` / `or` and `and.then` / `or.else` stay** (owner:
"`a`"): offered (a) unchanged, (b) swap which gets the plain word, (c)
only the skipping forms, design claude first leaning (c); the owner: "Why
does the other exist?" Design claude gave 2026-09-25's reasons (the code
does not lie, constant-time comparisons, every check reporting) and two
more (a mask's `and` works lane by lane; a type may define `and`, never
`and.then`), withdrew (c) and recommended (a). **The newline piece is `newln`, not `\n`** (owner: "`a`. I think it
should change from `\n` to `newln`. If `\` means escape, `\n` could read
as escape the letter `n` to consistent people"): plain quotes stay exact
and `format` keeps its escapes (option (a), over (b) a word, which this
is, and (c) C's escapes in plain quotes), and the piece outside quotes is
**`newln`**, matching `auto_newln`: `print.useful["Hello, World!",
newln];`, `"Hello" ++ newln`. Inside a format, `\n` stays, being format's
own escape. Design claude's reading: `\n` outside quotes is refused naming
`newln`; it was the only piece of its kind. No `tab` piece beside it (owner: "Isn't tab for
format?"): a tab is `format`'s `\t`. Design
claude's readings, not asked: `bin` and `deci` stay in the base (hardware,
or hardware on POWER and z); `int128` / `uint128` stay (standard, two
native operations each). Not asked yet: which cores they go to; what
deprecating `#strict` means (warned now, refused later?); SIMD, ranges and
`test fn`. The owner then: "Feeling like a syntax revision right now,
likely not gonna be removing, but adding for clarity". **Later: `deci` leaves the base too; `bin` stays**
(owner: "What has more chance of not having native? `bin` vs `deci`.
Remove the more"). Asked first "Can't you just write ASM for types?",
design claude: the maths yes, the type no (its size, its registers, what
LLVM optimizes and what Precog proves are the compiler's; a core, being
a compiler mod, can add a real type). Then: `bin` has hardware on nearly
every CPU built for (ARM64, x86-64, RISC-V with F/D; not the smallest
microcontrollers), `deci` only on IBM POWER (from POWER6) and z, software
everywhere else. So **`deci32` / `deci64` / `deci128` move to a core**,
which one not yet asked; design claude's reading, not asked: the
2026-09-29 layout ruling (DPD on POWER and z, BID elsewhere) goes with
it. **SIMD says its width: `simd128[int32]`, `simd256[int32]`,
`simd512[int32]`**, the lanes following from the element (owner: "`a`,
so the rule is even width looks like it should be dot narrowed, we just
don't narrow them"). Asked whether SIMD stays in the base (128-bit is
required on every ARM64 chip, NEON, and every x86-64 one, SSE2; LLVM falls
back to scalar code elsewhere), the owner asked which chips are not
128-bit (AVX/AVX2 256, AVX-512 512, SVE 128 to 2048, RISC-V V up to the
chip) and: "I think the syntax should be able to say which it is".
Design claude offered `simd.128[int32]`, lanes as today, or the
instruction set (`simd.neon`); the owner: "If bit needs narrowing there,
why doesn't types?", so the width is glued as in `int32`. **The rule (the
owner's): a width is part of the name, glued on, never a `.` narrowing,
even where it looks like one.** **Then undone: SIMD keeps its lane
count, `simd[int32, 4]`** (owner: "`a`"): asked how a mask says its lanes
under widths (`simd128[bool]` cannot: a `bool` has no width to divide
128 by; `mask128[int32]` offered), the owner: "Actually, maybe we should
consider `The lane count`". Offered (a) lanes as today, masks
`simd[bool, 4]`, the width following (4 × 32 = 128) and one lane count
naming one width; (b) widths with `mask128[int32]`; (c) both written.
Design claude recommended (a). So `simd[int32, 8]` is the 256-bit vector,
refused where the target has none or chosen by `on.machine` (design
claude's reading); the owner's rule on widths stands for any later type.
**SIMD stays in the base** (owner: "`a`"), like `bin`: 128-bit
vectors are required hardware on every ARM64 and x86-64 chip. **A vector
the target lacks is refused; a fallback is written by hand** with
`on.target` / `on.machine` and `else` (owner: "if handwritten could
already do auto, then handw"). Asked "should it refuse when no supported
hardware?", the owner asked whether `otherwise` helps (no: it handles a
call failing while running; missing hardware is known before, and the
alternative is other code, which `on.machine … else` already gives), then
"I think there should be a way to select fallback, or just refuse".
Design claude offered `simd.fallback[…]` (LLVM's fastest: two narrower
halves, or lane by lane where the chip has no SIMD), `simd.split[…]`
(halves only) or both; asked whether the hand-written way can do what
the automatic one would (yes: two 128-bit operations, or a plain loop, in
the `else`), the owner chose that. So no `.fallback` or `.split`. (Asked whether `on.machine` should be an `if`, design claude
recommending it stay its own word, the owner: "nvm": unchanged.) **Range
types move to ASP** (owner: "move"): `int32[0, 150]`, `int32[1, max]`,
since bounds need ASP's numbers-as-values (without it the oracle already
refuses a literal in one as undecided) and are not hardware. **Length
bounds go with them** (owner: "`a`"): `str.utf8[1, 40 bytes]` and
`list[int32][0, 100]` are ASP's too, all bounds in one place, over (b)
each beside its type (stdkit's lists, the text's owner); a bound counted
in graphemes also needs the Unicode text core. **`test fn` and `must` move to a core of their own** (owner:
"yea"): asked whether `test fn` stays in the base, moves to a core, or to
stdkit, design claude first argued kernels need tests; the owner: "I
mean, there could be specilized kernel building cores", so a kernel core
may bring its own testing instead. It cannot be stdkit's (a Limb adds no
syntax). Design claude had twice been wrong, saying `check` keeps
run-time checks (it asks Precog to prove a guardrail at compile time in
every build) and that tests use it (they use `must`, 2026-09-29); the
owner had asked "I though `check` was IEG's" (`check` is ASP's,
`no_check` IEG's) and "`check` only does runtime with debug build,
right?" (no). **The core is TAS, "Test And Shit"** (owner: "`tests`",
then, interrupting, "I meant `TAS` (This And Shit)", then "SHit, it's
Test, not This, mb"). **Its registry ID** (owner: "gen ID now"),
made as the other cores' were (`openssl rand -hex 32`): `63baa4cf89d8b6df6adceb812df005b30ea2771fd56f2117d0246ef911de21aa`,
first version `2026-10-07T18:08:59+07:00`. **`brain16`, the non-standard integers and `deci` go to a core
each** (owner: "`b`"), over one core for all three; design claude
recommended separate cores, since machine learning, cryptography and
money rarely want each other's numbers or software maths. **Their names**
(owner: "1. AI core 2. WSMW (Weirdly Small And Massive Widths) 3. Money
core"): **AI** for `brain16`, **WSMW, "Weirdly Small And Massive
Widths"**, for the non-standard integers, and **Money** for `deci`
(design claude's reading of "AI core" and "Money core" as the names AI and
Money). No registry IDs yet. **For the first public release, Money is covered and AI
is not** (owner: "That's actually a lot of work, money & AI. So, for
first public release, we only have to cover money"). **WSMW is in the
first release too** (owner: "First release."), after asking what it
helps with (odd widths matching audio, pixels, registers and protocols;
huge ones for cryptography, Ethereum's `uint256`, exact big numbers). **`int128` and `uint128` move to WSMW** (owner: "Yes"),
replacing design claude's earlier reading that they stay. Asked about
their hardware (no 128-bit registers anywhere; add, compare and
multiply are pairs of 64-bit operations, division a library call), then
"I mean, do the low level shit really use int128?" (rarely: 128-bit
atomic compare-and-swap, which is an atomics feature rather than
arithmetic, a multiply's high half, some cryptography; Linux's `u128` is
opt-in per architecture). So **the base keeps the register sizes, 8 to
64 bits**. **They keep their plain names, `int128` / `uint128`**
(owner: "a"), over `not_std.int128`: 128 is a standard width, living in
a core as `deci64` does in Money. **Unicode text goes to a core of its own** (owner: "`a`"):
`str.utf16`, `str.utf32`, `char.point`, `char.grapheme` and the Unicode
tables, over joining WSTN (names), so a program may print Thai or emoji
with English names. **It is UniTect** (owner: "UniTect, spelled like
that. C replaced X for code"): "UniText" with the X made a C, for code. **The case marks go to a core of their own** (owner: "`a`, name
this one yourself"), over joining UniTect (case marks are about names, not
text) or WSTN (they serve English names too); it carries the folding
rules it needs. **Design claude named it CAPS, "Capitals Are Pretty
Similar"**, at the owner's word. **Their registry IDs** (owner: "Gen"), made as the other
cores' were, first version `2026-10-07T20:22:24+07:00`: AI `8c723ccae25a719cd66eb44e95aeeef0557acec1467624ab91092c4ada766f21`; WSMW `0fb49902b57b8bc0b72caf424d80e17d035038e2cac58292d92f18b792d205a0`; Money `d7b62f4448a5b23d792a25bcdba89220e02691fb5e27cf18640ee491d049df0f`; UniTect `137a9a35afc69ec6ee796bc39eeefc75680db17ce8a04443126a492812a0ae96`; CAPS `90d0813dc72a0c7d54a4ded7a2c9577e77f0036958b245ab01808f9a8568bd01`. Not yet
asked: SVE's and RISC-V's run-time lengths.

### 2026-10-07 — No name "Signatar": the language is CyborgPL, and with no cores it is "base CyborgPL" (owner; recorded, not to be applied yet)

With no codextras and no dialects, the owner: "But should Signatar even
have a name anymore? Could just say base CyborgPL." Design claude agreed
(the name told the language apart from other codextras, "Signatar
CyborgPL" "like how they say BIOS for UEFI"; with one language a second
name only confuses) and proposed **CyborgPL** for the language and
**base CyborgPL** for it with no cores, a description like Rust's
`no_std`, rules worded "Signatar's" becoming "CyborgPL's" or "base
CyborgPL's". The owner: "Yeah. Record, don't implement yet. I may change
my mind." So **the rename is decided but not applied**: design notes, the
oracle, examples and CCA keep "Signatar" until the owner says go (about
130 places outside this file: design 28, the oracle ~37, CCA's compiler
55 and `PLUGINS.md` 36, CCA's crate `signatar` included); entries here
keep their words as history.

### 2026-10-07 — No dialects after all: Signatar is the language again, cores customize, anything deeper is a fork (owner)

Asked how a dialect would tailor optimization, design claude offered MIR
passes run before the ownership check and Precog, and LLVM passes after
it (trusted); the owner: "I honestly want LLVM passes too, any other way
to do it better like that?" Design claude offered tuning LLVM's own
passes, and custom LLVM passes checked by translation validation (RRx2-TV's
way), recommending both in final builds. The owner: "Or just fuck the
Dialect shit, and just let them fork so they could do whatever they
want". Design claude agreed: with no use across dialects a dialect is
nearly a fork already, and a dialect needs a stable AST, a MIR pass
interface and an LLVM pass interface kept stable for dialects nobody may
write; a fork may do anything under Apache-2.0 WITH LLVM-exception but
cannot be called CyborgPL (trademark) and merges our updates itself.
Asked whether to go back to the morning's design, the owner: "Yeah". So
**there are no dialects**: the two dialect entries below are undone, and
**the morning's design holds**: no base below the language, the language
is the lowest level and does the base's work, cores add words and
features and **a core may take powers away** (owner: "`a`", the game
engine's restricted setup), and **anything deeper, its own syntax or
optimizer, is a fork**.

### 2026-10-07 — (undone the same day: no dialects) Dialects: between the base and cores, each brings the syntax; Signatar is the base, CyborgPL the official dialect; no use across dialects (owner: b)

Right after there was no base, the owner: "We might wanna add a category
above core, but below Signatar, in rank. Btw, you could just call
Signatar as `base` or whatever". So **Signatar is the base**. Design
claude proposed the category's authority: it limits cores and everything
below (takes powers away, requires or forbids cores), so a game engine's
restricted dialect lives there and outranks a core that grants what it
takes away; adding no syntax; one per program. The owner: "It's called a
`dialect`. I think that the syntax should come from a dialect. While
Signatar, the base, is more like the JVM. You know?" So **a dialect
brings the syntax**, and **Signatar, the base, holds the meaning**: types,
ownership and borrows, Precog, powers, loading, the hand-off to LLVM.
Asked whether Limbs in different dialects may use each other, as Kotlin
uses Java's libraries, (a) yes, each `CCA.cplr` naming its dialect,
recommended, or (b) no, a program is all one dialect, the owner: "`b`,
we're not aiming for cross-compatibilty here. Because, that may make it
less customizable. And no shit, the official dialect is CyborgPL." So
**no use across dialects**: a program and every Limb in it are one
dialect; **the official dialect is CyborgPL**, today's syntax. Design
claude's reading, from its table, not separately confirmed: a dialect
also takes powers away and requires or forbids cores, which moves the
morning's "a core may take powers away" up to the dialect. A consequence,
design claude's: a program in another dialect has no std unless that
dialect brings one.

**Cores bring meaning, and may bring words too** (owner: "C is great").
Asked whether cores may still add syntax now that it comes from a dialect
(today IEG's `!check`, MCPS's `concur`, Genny's `<T>`), the owner first
asked the cost: design claude, none at run time (the base exists only
while compiling; the output is LLVM's native code, the same for two
spellings), a small, unmeasured hand-over per file at compile time, tiny
beside Precog's proving. Then which has more useful possibilities: (a)
cores bring meaning only, every dialect spelling it; (b) cores bring words
only to the dialects they name; (c) both. Design claude recommended (c):
add-ons mix and match (a units core's `5 meters`, a testing core's
`expect`) without a dialect for each combination, as compiler claude's
plugin hooks (Q1) already allow, and a game dialect still reuses any
core's meaning in its own spelling; the dialect stays in charge, since it
may forbid a core. So **a core brings a feature the base understands,
which any dialect may spell, and may bring its own words for the
dialects it names** (`for para` for CyborgPL; a game dialect writing
`every 'x' in 'xs' at once`, say). Design claude's reading, not asked:
MTM and WSTN stay cores that bring words to CyborgPL, which (c) already
fits; and where a dialect takes away a power a core grants, the dialect
wins, outranking it.

### 2026-10-07 — No base and no codextras: Signatar is the lowest level; a core may take powers away (owner: a)

Asked what the base does (reads the files, loads the codextra and cores,
enforces the grants, hands the codextra's LLVM IR to LLVM unread; no
syntax), the owner: "Tbh, there's no reason to create a language on this
base. So, could we remove it, and let Signatar be the lowest level?"
Design claude: the base's four jobs stay, as Signatar's; nothing changes
in speed; what goes is the codextra category and the rulings worded
"another codextra chooses its own". The owner: "Is there really a reason
for a lang to build on this base? It's not like it's the JVM, or
something that major." Design claude: no; the JVM gives a runtime, a
collector, a JIT and a shared library world, while the base gives only
what LLVM already gives, and core loading for cores written for
Signatar. The owner: "True. But before you remove codextra. How would
say, a game engine makes their CyborgPL dialect that's restricted, so
their games don't need verification?" Design claude: where the engine is
the program and games are limbs it loads, `deny` (built) does it; where
each game is its own program, only a codextra above it could restrict
it, so offered (a) **a core may take powers away too**, not only grant
them, the engine shipping a core its games list and its store requiring
it, Signatar still setting the limits no core lifts, recommended; (b)
nothing new, a store reading the build's power ledger. The owner: "`a`".

So: **there is no base**, and **"codextra" is no longer a category**:
Signatar is the language and the lowest level, and does the base's work
(reading files, loading cores, enforcing grants, handing IR to LLVM).
Signatar defines which powers exist and limits what cores may do; **a
core may grant powers and may take them away** from the programs and
limbs built with it; a program still only takes powers away. Rulings
that said "Signatar's rule; another codextra chooses its own" are now
the language's rules. Not asked: how a core spells what it takes away,
and which wins when one listed core grants a power another takes away.
Design claude's own question, "by 'don't need verification', the engine
not reviewing each game, or skipping Precog's proofs?", was not answered.

### 2026-10-07 — The revision and the slimmer base are built in the oracle (owner: "Build now")

Everything decided today in the syntax revision and the slimming is built
in the oracle, in seven commits (spellings, labels, cases and printing,
signatures, `#strict`, the new cores, two fixes), every example and test
rewritten, 692 reject tests compared one by one against their errors
before the build so each still fails for its own reason (the ones that
tested MTM's spellings, `#strict`'s rules or the arrow variants became
tests that the old form is refused). The old → new table is
`design/revision-2026-10-07.md`; the design notes' code blocks are
converted (their comments, which may quote the owner, untouched), and each
affected note points to the table. The rename of Signatar is not applied
(owner: "Record, don't implement yet").

Design claude's readings while building, not asked (open questions):
- An untyped variadic (`...'parts'`, each argument keeping its own type,
  2026-09-19) is still allowed under "types required on every function";
  requiring a type would remove that feature.
- `otherwise 0`, the short form with no cases, takes no `is`.
- A nested `match` that is a case's value says `is match …`, the lone
  `match` in a case being a statement; this replaces 2026-09-29's "the
  outer match decides", which `is` everywhere made moot.
- `.stdout` may still be written; `.stderr` is the only one needed.
- A `std` prefix stays reserved beside `stdkit` (it names the built-in's
  old spelling).
- Counting in points or graphemes (`str.graphemes`, a `40 graphemes`
  bound) needs UniTect; an exact decimal literal (`*0.1`) needs Money.
- Two bugs found today are fixed: an undeclared type name in a function
  nothing calls is refused, and a vector in an `on.target` branch ruled out
  for this target is not judged against it.

### 2026-10-07 — The inline languages' main names are `inline.ASM`, `inline.LL`, `inline.MSL`; the whole names go to MTM (owner: a)

```
!check inline.ASM { neg x0, x0 } with [ … ];   # Signatar's
inline.LL { … }                                # IEG's: LLVM IR in place
inline.MSL threads 'n' { … } with [ … ];       # the Metal core's
```

Asked to see the inline names, main spellings not the others, design
claude listed 2026-10-06's whole names (`inline.Assembly`,
`inline.LLVM_Intermediate_Representation`, `inline.Metal_Shading_Language`,
with MTM's `inline.ASM`/`inline.S`, `inline.LLVM`, `inline.MSL`). The
owner: "Main should now be `inline.ASM`, `inline.LL`, `inline.MSL`".
This replaces 2026-10-06's "the entire language's name when inline".
Asked what the displaced spellings become, (a) all MTM's, recommended,
or (b) refused naming the main, the owner: "`a`". So MTM has
`inline.Assembly` and `inline.S` for `inline.ASM`,
`inline.LLVM_Intermediate_Representation` and `inline.LLVM` for
`inline.LL`, and `inline.Metal_Shading_Language` for `inline.MSL`.
Design claude noted that `.ll` is LLVM IR's text file extension.
`extern.C` and `layout.C` are unchanged. The Nvidia and Vulkan cores'
inline languages are still unnamed, and `inline.C` is still only
possible. In the oracle (which reads only `inline.ASM`; `inline.LL` and
`inline.MSL` are refused as not read yet): reject tests
`mtm_inline_assembly`, `mtm_inline_s`, `inline_ll_not_built`,
`inline_msl_not_built`; every example and test uses `inline.ASM`. While
listing them, design claude also found and fixed its own 2026-10-06
rewrite of the design notes, which had left `inline.LLVM` in
`design/cores.md` and made `design/ffi.md` read "`inline.Assembly` is
another name for `inline.Assembly`".

### 2026-10-07 — `simd.sum` is a plain number, checked where it lands; `simd.sum.lane` and `simd.sum.wrap` work at the lane's type (owner: yea)

```
'v': simd[int32, 4] = simd.fill[1200000000];
's': int64 = simd.sum['v'];         # 4800000000: worked out in full, checked where it lands
't': int32 = simd.sum.lane['v'];    # stops: the sum is checked at int32
'w': int32 = simd.sum.wrap['v'];    # 505032704: wrapped at int32, never a check
```

Check claude found it (seed 30022): design/simd.md (2026-09-22) says `sum`
"can overflow T", and CCA stopped; the oracle worked it out in full and
ran. Design claude asked (a) checked at the lane's type or (b) worked out
in full, checked where it lands, first recommending (a) after the
2026-09-29 (b) ruling that lanes are checked at their type. The owner:
"Would `b` defeat the purpose of simd>" Design claude: no: the lanes stay
their size through the loop, the sum is done once and its answer is a plain
number, not a lane; ARM adds four int32s straight into an int64 in one
instruction (`SADDLV`); so it changed its recommendation to (b). The owner:
"There should be a way to do the other option too". Design claude offered
(b) as the default, `simd.sum.lane` for (a), and `simd.sum.wrap`, the
chip's own `ADDV`, wrapping as `plus.wrap` does. The owner: "yea, you sad
no check, so would it need `!check`?" Design claude: no; `plus.wrap` is
"always defined, never a check, nothing to `!check`" (2026-09-20), and
wrapping is the meaning, not a skipped check. `simd.max`/`min` cannot pass
their type and have neither form. Design claude's readings, not asked:
`simd.sum.lane` checks the total, not each partial sum (the order the
chip adds lanes in is not the program's, and a wrapped partial sum comes
back); `.wrap` takes integer lanes only, as `plus.wrap` does. In the
oracle: `examples/simd_sum_forms.cpls`, reject test `simd_sum_lane_past`.

### 2026-10-07 — A whole quotient that rounds to zero is `0`, never `-0`; `div.ceil.ieee` keeps it (owner: b, a)

```
let 'q': bin64 = 0.5 div.ceil (-1.75);    # -0.2857… rounded up: 0, not IEEE's -0
```

Check claude found it (seeds 25502, 26092): on a bin or `brain16` the
oracle gave IEEE's `-0`, CCA gave `0`, a `deci` has no `-0`, and no ruling
covered it. Design claude offered (a) keep `-0` as the hardware does, (b)
always `0`, recommended, (c) the dividend's or divisor's sign; asked the
cost: one floating-point add of `+0.0` after the rounding, none where
Precog proves the answer cannot be `-0`. The owner: "`b`, but is there a
way to do like hardware?" Design claude offered (a) a narrowing on the
operator, `div.ceil.ieee`, std's, beside `div.trunc.wrap`, free, written
where wanted, recommended; (b) no new syntax, IEG's raw float operations
or `inline.Assembly`. The owner: "`a`". So **`div.floor.ieee`,
`div.ceil.ieee`, `div.trunc.ieee` keep IEEE's `-0`**, on a bin or `brain16`
only (an integer or deci has none: refused naming the plain operator).
Design claude's readings, not asked: `div.near` takes no `.ieee` (on a bin
it is ordinary division, no rounded zero), nor does `mod.X` (a zero
remainder already takes the divisor's sign). In the oracle:
`examples/zero_quotient.cpls`, reject tests `ieee_on_int`, `ieee_on_mod`. **Then no `-0` anywhere**
(check claude, seed 30208: a zero `times` a negative, and `-'b'` of a
zero, gave `-0` in the oracle and `0` in CCA): asked (a) no `-0` in any
bin or `brain16` result, every operation that could make one adding
`+0.0` (free where Precog proves the answer is not zero), the `.ieee` forms
keeping it, or (b) only rounded quotients, the owner: "`a`". Asked "what's
correct?": by IEEE 754, `-0` is a real value, kept so that `1 / -0` is
negative infinity and a result too small to hold remembers its sign; in
maths zero has no sign. CyborgPL's bins hold no infinity at all
(2026-09-29), so `-0`'s main use is gone, and plain `0` is the better
answer here.

**The `.ieee` family** (owner, 2026-10-07). Design claude asked whether
`.ieee` should go on `times`, `minus` and `div.near` too, and noted that
negation would be `-.ieee 'x'`, which reads oddly. The owner: "yea, and
`-.ieee 'x'` defeats the purpose of a symbol". Design claude offered the word
CyborgPL already has for negation, `neg` (`op neg`, precedence levels),
narrowed: `neg.ieee 'x'`, with `plus.ieee` in the family as well (`-0 plus -0`
is `-0` in IEEE). The owner: "yea". So: **`plus.ieee`, `minus.ieee`,
`times.ieee`, `div.floor.ieee`, `div.ceil.ieee`, `div.trunc.ieee`,
`div.near.ieee`, and `neg.ieee 'x'`** keep IEEE's `-0`; plain `-'x'` never
gives one. This replaces design claude's earlier unasked reading that
`div.near` takes no `.ieee`.

**`mod` and `pow` too** (owner, 2026-10-07). The oracle first refused
`.ieee` on them (design claude's reading; its reason for `mod`, that a zero
remainder takes the divisor's sign, no longer held once `3.0 mod.floor -3.0`
became `0`). Asked "Should every operator that can make a `-0` have an
`.ieee` form, `mod` and `pow` included?", recommended yes, the owner:
"yea.". So **`mod.floor.ieee`, `mod.ceil.ieee`, `mod.trunc.ieee` and
`pow.ieee`** (bare or `std::pow.ieee`) keep it as well; `mod.near` has none
(on a bin it leaves nothing over). Design claude's readings, not asked:
rounding where a value lands (a
bin64 into a bin32), std's `round`/`floor`/`ceil`/`trunc` and a
conversion into `brain16` never make a `-0` of a number that was not zero,
and keep one the `.ieee` forms made; `simd.max`/`min`/`sum` give `0`. In
the oracle: `examples/zero_everywhere.cpls`, reject tests `ieee_on_mod` (now
`mod.near.ieee`), `neg_ieee_on_int`, `plus_ieee_on_int`.

### 2026-10-06 — Without IEG nothing needs `!check`: the marking rule is IEG's (owner: yes)

The owner: "If `inline.Assembly` comes with Signatar, shouldn't it not
require `!check`? And only require when IEG is here?" Design claude: that
is this morning's ruling ("Signatar marks nothing … IEG brings the
markers … and the rule that raw memory and unproven code sit under
`nocheck`"), which the oracle and CCA had never applied, and asked whether
the same holds for the rest of that family: C calls, `ptr.read` /
`ptr.write` / `ptr.cast`, C variables, a non-literal `print.format`. The
owner: "yes". So in a Limb without IEG these are written plainly; where
IEG is listed (and in a lone file, which has every core) each sits under
`!check`. `examples/signatar_hello_c` (Hello World on Signatar alone,
through C's `write`), reject test `asm_signatar_unmarked`.

### 2026-10-06 — `!std.int24`, not `!std_int24`: "not standard", narrowed with `.` (owner: yea)

```
'n': !std.int24       # was !std_int24
'm': !std.uint4       # MTM's Rust-style: !std.u4
```

The owner, looking at the `!` forms ("`!` is becoming as iconic as
`:=`"): "I don't think there's any other way to write `!std`", which
design claude read as `!std` being "not standard" too, a `!` that means
"not" like the rest; then: "should it be `.` instead of `_` now?" Design
claude: by the owner's rules, `_` joins the words of one name
(`LLVM_Intermediate_Representation`, `NO_FUCKING_CHECKS`) and `.` narrows
(`str.utf8`, `div.floor`), and `!std_int24` is a category narrowed to one
member; MTM's twins become `!std.i24` / `!std.u24`, and `!std_int24` is
refused naming `!std.int24`. The owner: "yea". In the oracle (type names
print `!std.int24` too); examples, tests and notes rewritten, the owner's
quotes kept; reject test `std_underscore_old`.

### 2026-10-06 — `synthia sync` writes the `CCA.cplr` files and the renamed copies, leaving the project ready for `cca build` (owner; relayed by synthia claude)

```
synthia sync     # checks the whole tree of path dependencies, writes the CCA.cplr files
cca build        # builds, knowing nothing of Synthia
```

`synthia sync` writes the project's `CCA.cplr` from `Synthia.ttsr`,
writes one for each needed Limb that has only a `Synthia.ttsr`, and makes
the renamed copies in `.synthia/`, after checking the whole tree of path
dependencies. Synthia claude offered (a) `synthia sync` (uv's word for the
same job), (b) nothing until `synthia build` exists, (c) `synthia write`;
the owner: "what"; re-asked plainly with the two-command example: "`synthia
sync`". `synthia build` would later run it before CCA. Built the same
day in Synthia: it reads hand-written `CCA.cplr` files (4093555) and
follows path dependencies with CCA's refusals (4acbd32).

**A needed Limb on this machine with only a `Synthia.ttsr` gets its
`CCA.cplr` in its own folder** (`limb.money.path = ../money` gives
`../money/CCA.cplr`, beside its `Synthia.ttsr`), over a copy in the
project's `.synthia/`; synthia claude recommended it (what a CCA user
does by hand, one copy of the code, and `money` then builds by itself
with `cca` too); the owner: "`a`". A hand-written `CCA.cplr` there is
never replaced (2026-09-30); `synthia takeover` in that folder hands it
over. Synthia also lists WSTN and MTM as official cores (ec97abb).

### 2026-10-06 — `>>` and `>>>` are other spellings of `-->`, and `<<` and `<<<` of `<--` (owner)

```
fn double ['n': int32] >> int32 { … }
std::print.weird.useful.stdout << double[3] <<< \n;
```

The owner: "I think we should add `>>` & `>>>` as alias for the arrows".
Design claude noted they are free (shifts are `shl`/`shr`, not C's
`>>`), asked about the twin (`<<`, `<<<` for `<--`), and that a Limb could
no longer name its own operator `>>`, since any run of operator
characters could be one; the owner: "yea.". `-->` and `<--` stay the main
spelling. **`<<>>` is another spelling of `<->`**, the both-ways asm
binding (design claude asked about that twin; the owner: "`<<>>` too").
**The long two-way arrow is the main spelling, `<-->`**, with **`<<<>>>`**
beside it, and **`<---->`** too (design claude asked, since `-->` and
`<--` are main and the two-way arrow was only ever `<->`; the owner:
"yea. Also, shouldn't there be `<---->` too? Two long arrows"). So the
both-ways binding is `<-->`, or `<->`, `<<>>`, `<<<>>>`, `<---->`. **And
the walled ones: `<--|-->`, `<-|->`, `<<|>>`, `<<<|>>>`** (the owner:
"`<--|-->` should be too, I think"; asked about the twins and how many,
nine two-way spellings with them: "`b`"). Seventeen arrow spellings in
all: four out, four in, nine both ways. **Signatar has the long arrows only**,
`-->`, `<--`, `<-->` (owner: "Signatar supports long, not short
anymore"), after design claude recommended one spelling per direction in
Signatar and the owner first leaned to the short ones, then saw both
side by side; the other fourteen come from a core. The owner asked
whether that is a separate core, "because it also includes `&` and
`&mut` (yes, no dot)". **A separate official core, MTM, "Maybe Too
Much"** (owner: "yea, all in one separate core. `MTM` (Maybe Too Much)"),
holds every other spelling of something Signatar has: the fourteen other
arrows; `&` / `&mut` for `lend` / `lend.mut` and `&'x': T` / `&mut 'x': T`
for `lent` / `lent.mut`; Rust's number names (`i32`, `u8`, `f64`, …);
`bfloat16` / `bf16` for `brain16`; `!std_i24` / `!std_u24`; and `inline.S`
for `inline.ASM`. Signatar keeps one spelling of each. The owner also:
"Signatar would use the entire language's name when inline instead", so
Signatar's `inline.` takes the language's full name: **`inline.Assembly`**
(Signatar's), **`inline.LLVM_Intermediate_Representation`** (IEG's),
**`inline.Metal_Shading_Language`** (the Metal core's); `extern.C` and
`layout.C` stay, C being the whole name; MTM keeps `inline.ASM`,
`inline.S`, `inline.LLVM`, `inline.MSL`. Words joined by `_`, every word
written out (owner, asked whether a dot narrows there: "Idk about dots,
man, do they really narrow?"; design claude: barely, so `_` for the
spaces; the owner: "`Low_Level_Virtual_Machine_Intermediate_Representation`
style, I think"). Design claude noted LLVM is no longer an acronym (the
project dropped "Low Level Virtual Machine"; LLVM is its whole name);
the owner: "Yeah. Welp, I mean, maybe not much of a VM. Mf, that shit
powers multiple langs", so `LLVM_Intermediate_Representation`. Not built.
**`!` is Signatar's main spelling of "not"** (owner: "`!` infront is the
main spelling of the things like that? Say `nocheck`, `!$`, etc", then
"yea"): **`!mut`** is main and `immut` moves to MTM; **`!check`** is main
and `nocheck` moves to MTM; `!$`, `!cheat` and `!==` already were. For
`NO_FUCKING_CHECKS` the owner: "would become `UB` main, if `UB` is
accurate"; design claude: not accurate (undefined behaviour follows any
wrong vouch, plain `!check`'s too; what this tier adds is trusting C's
answers), offering `!check.C`; the owner: "The fuck is `!check.C`?",
then **"keep `NO_FUCKING_CHECKS` main"**. Not built. **`and`, `or`, `not` stay
words** (std's operators, as `plus`), **and MTM brings `!` for `not`**
(owner: "stay words, while MTM has `!` for not"); design claude had
floated C's `&&` / `||` for MTM too, which the owner did not take up. **The value's `!` is spaced: `! 'ready'`** (owner: "Why not with a
space, `!` without space reads as a rule"), and **a glued `!'ready'` is
refused** (owner: "`a`"), naming the spaced form: a glued `!` marks a
rule (`!mut`, `!$`, `!check`, `!cheat`), a spaced one negates a value.
Then: **`!` is the main spelling of `not`, and the word `not` moves to
MTM** (owner: "Make `!` the main spelling for `not`, so them kernel
makers won't trip `!check` as `check`"): one `!` meaning "not" across
the language, so `!check` reads as "not check" to anyone who knows it.
**The design notes are rewritten to the main spellings** (`!check`,
`!mut`, `! 'x'`, `inline.Assembly`), the owner's quotes and the lines
naming MTM's spellings kept as written (owner: "Bro, of course that my
exact words/rulings"); this log keeps the spelling of each entry's day.
**`not.bit` stays a word** (owner: "`a`"), with the bitwise family
(`and.bit`, `or.bit`, `xor`, `shl`, `shr`), all std's. Not built. **MTM adds
other spellings for any layer, each working only where its owner is**
(design claude's reading, the owner: "yea"): `immut` for Signatar's
`!mut`, `nocheck` for IEG's `!check` (the markers are IEG's, 2026-10-06;
design claude had called `nocheck` Signatar's and corrected it), `not` for
std's `!`, and the arrows, `&`, Rust's names for Signatar's own. The
owner: "the other lang writers would be fucking confused, even though
it's actually consistent. Which is fine".
**MTM's ID** (owner: "gen ID now"): `9c6c773027ae873a736d2e9b2dc398fd4807a7df6cf43b7e233c3447af3d6a56`, first version `2026-10-06T19:18:54+07:00`.
In the oracle; `examples/chevron_arrows.cpls`, reject tests
`op_chevron_reserved`, `asm_both_chevrons`.

### 2026-10-06 — Signatar's names are English only; names in other scripts come from a core (owner; built in the oracle 2026-10-07)

Asked whether Signatar supports every language in its variables (the
oracle accepted anything inside quotes, `'ไม่'`, `'😀'`; bare names took
Unicode letters and digits, so `fn ไม่` and `fn नमस्ते` failed on their
combining marks; none of it ever decided), with design claude
recommending Unicode's identifier rule (UAX #31) for bare names, the
owner: "Bare names should be English only. But should the crazy names
come from another core?" Design claude: yes, a core may add syntax and
cores make dialects, so Signatar stays plain and a core opens names to
every script (or emoji); asked whether quoted variable names are English
only too: "English only too, crazy ones from the core". So **every name
in Signatar, bare or quoted, is English letters, digits and `_`**
(design claude's reading of "English": ASCII `a`–`z`, `A`–`Z`); **names
in other scripts come from a core**. Text values (`"สวัสดี"`) are not
names and stay any Unicode. With Signatar's names ASCII, 2026-10-06's
Unicode case folding matters where that core is used. **It is a new
official core, WSTN, "Wanted Some Thai Names"** (owner: "`a`, `WSTN`
(Wanted Some Thai Names) is the name"), over leaving it to anyone.
**WSTN allows every script, and emoji** (owner: "`b`"), over (a) Unicode's
identifier rule (UAX #31) alone, which design claude recommended (emoji
may be several code points, and two can look the same and differ). The
owner: "don't our err msgs use graphemes? And what cost does a var name
using an emoji do?" Design claude: the messages were made plain text,
with no carets under the source, partly because Thai graphemes are not
code points (`diagnostics.md`, 2026-09-22), though what unit a message's
column counts is not stated there; a name costs nothing at run time
(names vanish when compiling, kept only as debug symbols). **WSTN's ID**
(owner: "yea", generated as the other cores' were): `e86b0df56e35b02b2ee91df6d89e63011564364d404ed7f5ea25d768d6d854d4`, first
version `2026-10-06T18:48:22+07:00`. **A message's column counts graphemes** (owner: "our error
messages are gonna be graphemes, unless you have a better one"); **the
JSON form counts in the unit the editor asks for, UTF-16 code units by
default, as the Language Server Protocol does** (design claude's
proposal; the owner: "yea"), so an editor's cursor lands right on Thai
and emoji lines. **This is Signatar's rule** (the owner, asked
whether Signatar's own messages count graphemes: "it's Signatar's, we
can't force the other codextras to use this anyways"): Signatar's
messages count graphemes, the Unicode tables living in its compiler
plugin, never in a program; another codextra chooses its own.

### 2026-10-06 — A Limb's writer may make its names case-insensitive, in its own code and as reached from other Limbs (owner; built in the oracle 2026-10-07)

Asked whether CyborgPL is case-sensitive (it is, in the oracle and CCA:
`Area` and `area`, `'X'` and `'x'` are different names; `Main` is not
`MAIN`; module, Limb and prefix names are lowercase only, for the file
systems; not recorded as a ruling, the owner: "No, not yet"), the owner:
"There should be a way to change to case-insensitive for what". Design
claude offered (a) names in code (variables, functions, types, fields,
cases), (b) keywords too, (c) names reached from other Limbs, and
recommended (a) alone. The owner: "`a`, and `c`. Would depend on their
respective writers". So **a Limb's writer chooses**; design claude's
reading: when a Limb chooses it, its names are insensitive inside it, and
other Limbs reaching them (`geometry::Area[…]` finding `area`) do so
insensitively too, since the choice belongs to the Limb that owns the
names. Keywords stay exact. The owner: "yea" to the reading, "but for
which too": **the switch says which names it covers, at every level the
`bare` marks have** (owner: "`b`"): by kind (`fn`, `struct`, `enum`,
`interface`, `const`, `case`, `field`, `var`, design claude's list), and
also on one item and on a module or file, over (a) kinds only in
`limb.cpls`, which design claude recommended. **Spelled `case.insensitive`
and its twin `case.sensitive`** (owner: "yea"), as the `bare` marks are:
`case.insensitive [fn, struct];` in `limb.cpls`, `pub case.insensitive fn
area […]` on an item, `case.insensitive;` as a file's first line. **The default is
`case.sensitive`** (owner: "`a`, but `case.sensitive` shouldn't be an
error"), over no default as for `bare` (every name, each local variable
included, would need covering): names are exact unless marked, and
writing `case.sensitive` anyway is allowed, never refused as saying
nothing. **Marks that disagree are an error** (owner: "`a`"), as for
`bare`, over the narrowest winning.
**Insensitive means full Unicode case folding** (owner: "`b`"), over ASCII
letters only: names may hold any Unicode letter (`'café'`, `Größe`), so
`É`/`é` and `ß`/`SS` match as Unicode defines; names are compared while
compiling, so it costs nothing at run time.

**Built in the oracle 2026-10-07** (owner: "build them now"). Design
claude's readings there, not asked:
- An item's mark covers its own name; a struct's fields and an enum's
  cases are covered by the `field` / `case` kinds, or by a file or block
  module mark.
- A name differing only in case from an insensitive one in the same Limb
  is refused ("they would be one name"), as is a mark before `pub`
  (`pub case.insensitive fn …` is the spelling).
- Keywords, std's words, the built-in types and `my.` stay exact.
- Local variables are covered by the `var` kind, or by a file or block
  module mark. A file's `case` mark and its `bare` mark may come in either
  order at the top.
- `op` is not one of the kinds, though an item mark may go on one.
- Other Limbs reach an insensitive name in any case through its path,
  bare where its `bare` mark allows, and through the fields and cases of
  its pub types.
- Full case folding is approximated as upper case, then lower case.

In the oracle: `examples/case_marks.cpls`, `case_file.cpls`,
`case_unicode.cpls`, `uses_case` (with `limbs/shout`); reject tests
`case_*`.

### 2026-10-06 — In `Synthia.ttsr`, every needed thing starts with `limb.` or `core.`; a core is listed there by ID and version (owner; relayed by synthia claude)

```
[my needed things]
core.asp.id = 4882d1f3bf7b11d59879e4921672f15e91369e25d55a10e3dcd24e9b3fbe676f
core.asp.version = 2026-10-06T18:04:29+07:00
limb.std.id = fff4302d…1936
limb.std.version = 2026-10-06T12:57:14+07:00
limb.money.path = ../money
limb.money.prefix = cash2
```

Synthia claude offered (a) a `[my cores]` section with `asp.id`,
recommended, or (b) inside `[my needed things]` as `asp.core = …`; the
owner: "inside `[my needed things]`". Confirming `asp.core` +
`asp.version`, the owner: "`core.asp.id`, I think, and
`limb.geometry.id`"; shown a whole section, noting it changes the ruled
Limb lines (`geometry.id` becomes `limb.geometry.id`): "yea". The old form
is refused, showing `limb.geometry.id = …`. Built in Synthia (776300d):
official core IDs only, `core.x.path` refused for now, std without ASP
refused in `Synthia.ttsr` itself, and the written `CCA.cplr` lists the
cores. Design claude ran the oracle on Synthia's output for
`examples/limbs/areas`: accepted, same program output.
`synthia-manifest.md`.

### 2026-10-06 — A core is listed by its registry ID, in `CCA.cplr` and `Synthia.ttsr` alike (owner: "ID"; relayed by synthia claude; not built)

Synthia claude asked how `Synthia.ttsr` lists cores ((a) a `[my cores]`
section with `ASP = yes` lines, (b) one line `cores = ASP, IEG`); the
owner: "Shouldn't it use ID?" Synthia claude noted that cores.md already
gives a core a registry ID (the "Fastmath" warning), that names repeat
and IDs do not, that a core runs inside the compiler, and that Synthia
can use IDs only if `CCA.cplr` does too; asked whether cores are listed
by ID in both manifests, the owner: "ID". Replaces the `cca::Core[name =
"IEG"]` shape (built in the oracle and CCA). **In `CCA.cplr` a core is
`cca::Core[id = "…64 hex…", version = "…"]`, with a version, as a limb**
(owner: "`a`"), over (b) the ID alone meaning the installed release:
a core changes the language, so a project pins the release it was
written for; CCA uses the installed copy when the ID and version match,
as for std. **The official cores' IDs and first version** (owner: "yea",
to doing it as for std: design claude ran `openssl rand -hex 32` once per
core and took what came out; the version is the release's date and time
with the offset, std's rule), every first version `2026-10-06T18:04:29+07:00`:

```
IEG      5d0e846b2a0f57f6fae20216e8d856d41b6257e0a6104aaf0bc55b57b44b4085
ASP      4882d1f3bf7b11d59879e4921672f15e91369e25d55a10e3dcd24e9b3fbe676f
Genny    c3e299d99a13567b2039c13fc17d0ca562be9d6f657f8c40a4774081983b20c9
MCPS     7d0ccfe6e41eed3a4d8ffe00ccaada9b89a4b9a3ea553a9d869cb6902e2ca75d
AutoGPU  67b2514aab0ffa5ca9c3ae48144498880466d68aa3b8b8f8e1082e6032c24c4d
Metal    3e7b81fec072c86d58538d10978aff1d42814c9e907f1259766851795cded9b7
Nvidia   beefcce9ed7cccf0d62f5a6e004c4bd53c6ae9e3a45c48883d3b1dc4be913361
Vulkan   63ff7ed8da5e0eead0164488e89553b240501f16d4be863ba6be1e46b51ec3ec
```

**Until cores can be loaded, any other ID is refused** (owner: "`a`"):
an ID is registered in the official registry (2026-09-29), so another ID
is someone else's registered core, the "unverified, used with a warning"
case (2026-10-05); with no plugin loading yet, the oracle and CCA refuse
it ("loading another core is not built yet"), and the refusal becomes the
warning once plugins exist. Over (b) accepting it with the warning now,
which would build as if the core were there. **A core never registered
is listed by `path`, as a local limb:** `cca::Core[path = "../mycore"]`
(owner: "`a`", over registered cores only), unverified, so warned, and
refused like an unknown ID until plugins load. Open: `Synthia.ttsr`'s spelling (synthia claude's
to ask).

### 2026-10-06 — `to.int8.bits[…]` takes any value whose bits fit, -128 to 255, and refuses the rest (owner: a)

`to.int8.bits[200]` is -56 and `to.uint8.bits[-1]` is 255 (both are an
8-bit pattern, read either way); `to.int8.bits[300]` is refused (it needs
9 bits). Over (b) the target's unsigned patterns only, 0 to 255, and (c)
keeping the low 8 bits, as C's cast and Rust's `as` do. Design claude's
reasoning: it matches the owner's refusal of too-wide literals, and -1
really is an 8-bit pattern. The owner asked what fits (an `int8` holds
-128 to 127, a `uint8` 0 to 255, the same 256 patterns read two ways),
noted the usual `to.int8[…]` stays checked and only a written `.bits`
reinterprets, and asked the cost: a literal costs nothing; a value known
only at run time costs one comparison, none when Precog proves it fits or
under `nocheck`, where (c) would have been free. The owner: "`a`". To
cut on purpose, the writer says so: `to.int8.bits['x' and.bit 255]` or
`to.int8.bits['x' mod.floor 256]` (300 gives 44, -300 gives -44 with
`and.bit`).

### 2026-10-06 — The root path is `my.`, not `limb.` (owner; not built yet)

```
my.helper[…]                # was limb.helper[…]
my.list.append[…]           # a Limb's own name, where a bare one clashes
```

Asked, after a Limb's own clashing name was ruled to be reached through
its root ("yea"), "but should it be `mine.` instead?". Design claude: for
`mine.`, it reads as "my own" and cannot be read as "some Limb"; for
`limb.`, it pairs with `pub.limb`; leaned slightly to `mine.`, leaving
`pub.limb` as it is ("public within the Limb" is another idea than "my
root"). The owner: "`mine.`" (recorded, 251e9dd), then, asked about the
root file: "Actually... Is it `my.`?" Design claude: "my" goes before a
noun and a path always has one (`my.helper`, "my helper"); Perl's `my`
declares a variable, but here it is always followed by `.`; recommended
`my.`. The owner: "yea". Reverses 2026-09-19's `limb.` (chosen over the
floated `std:to` / `std:::to`). About ten uses in the examples and tests.
**The root file stays `limb.cpls`** (owner: "`a`, but I thought it was
`main`"), over renaming it `my.cpls`. It was never `main`: `limb.cpls`
was chosen 2026-09-19 over having no root; `MAIN { }` is the entry
block, which lives in it.

### 2026-10-06 — A Limb says whether its path is required or optional, per item, per module and per kind (owner: "all"; not built; reverses 2026-09-23's "drop bare")

Asked how std's list, map and text operations are written now that they
are std's ((a) bare, refused without std, as std's operators; (b)
`std::list.append[…]`), the owner: "A limb should be able to say if path
is required, or optional, and for what". Design claude flagged that this
reverses **2026-09-23's "drop bare"** (everything from another Limb
carries its path, only operators bare), which had itself reversed
2026-09-19's "the Limb maker decides bare reachability", and asked what
"for what" covers: (a) per item (`pub path.optional fn …`, *sketch*),
(b) per kind, once for the Limb ("my operators and my `list.` functions:
optional"), (c) per module, recommended. The owner: "all". So the
writer of a Limb can mark the path optional at any of the three. **Marks
that disagree are an error** (owner: "`c`"), over (a) the narrowest
wins, recommended, and (b) "required" always wins: a Limb marking the
module `list` optional and its item `list.clear` required is refused,
and the writer makes them agree. **The word is `bare`, as 2026-09-23's
`pub bare fn pow […]`** (owner: "yea"): `pub bare fn append […]` for an
item, `pub bare module list { … }` for a block module. **A file module
says `bare;` on its first line** (owner: "`a`", after asking how it
looks), over a list in `limb.cpls` or in `CCA.cplr`: every `pub` thing in
`list.cpls` may then be written without `std::` (`list.append[…]` or
`std::list.append[…]`). **A kind is marked in `limb.cpls` with a list:
`bare [op];`, `bare [op, fn];`** (owner: "`a`"), over a `bare` field in
`CCA.cplr`: a plain `bare;` means this file, a `bare [ … ]` list means
those kinds everywhere in the Limb and is allowed only in `limb.cpls`;
the kinds are the keywords `fn`, `op`, `struct`, `enum`, `const`,
`interface` (`const` dropped the same day, below). **A Limb may also forbid the path** (owner: "Limbs should
be able to forbid"), so three choices: required (the default), optional
(`bare`), forbidden. Asked after design claude said a path was always
allowed, so a clash always had the fix "write the path" (glyph operators
like `~=` already have no path, 2026-10-02). **Forbidden is
`bare.only`** (owner: "yea"), the `.` narrowing `bare`, at every level:
`pub bare.only fn …`, `pub bare.only module …`, `bare.only;`,
`bare.only [op];`; writing the path to one is refused ("`list` forbids
its path: write `list.append`"). **Two Limbs that both forbid the path
to one name are an error as soon as both are depended on** (owner:
"`a`", over (b) only where the name is used, which design claude
recommended), fixed as a prefix clash is, by editing one Limb's source
(its `bare.only` to `bare`) or by Synthia doing it: the owner, asked,
"Aren't you suppose to edit a limb's source code to change its prefix,
unless you use Synthia?" (2026-09-29: "Just edit the source code").
**Optional is `bare.optional`, and a plain `bare` is refused** (owner:
"Shouldn't bare `bare` be an error, because it doesn't say if it's
optional?", then "yea"), as a plain `div` is: "say `bare.optional` or
`bare.only`". So every earlier `bare` here reads `bare.optional`:
`pub bare.optional fn …`, `bare.optional;`, `bare.optional [op];`.
**Two Limbs' `bare.optional` names clash only where one is written
without its path** (owner: "`a`"): "write `std::list.append` or
`fastlists::list.append`"; depending on both is fine. **A Limb's own
name does not win over them** (owner: "no", to design claude's "your own
wins"). **There is no default** (owner: "I don't think there should be a
default"): asked whether a Limb's operators follow a default of "path
required" ((a), recommended) or stay optional as before ((b)), the owner
ruled out a default altogether, so every `pub` thing a Limb offers is
covered by a mark (on it, its module or its kind) saying required,
optional or forbidden. **Required is `bare.never`** (owner: "yea"), over
`bare.no` and `path.required`: "never bare, sometimes bare, only bare". A
Limb wanting every name on its path writes `bare.never [fn, op, struct,
enum, const, interface];` in `limb.cpls`.
**std marks everything `bare.optional`** (owner: "optional for all"),
over design claude's split (operators and `list`/`map`/`str` optional,
the rest `bare.never` so `std::print` shows its origin): `print.useful
.stdout[…]` and `std::print.useful.stdout[…]`, `list.append[…]` and
`std::list.append[…]`, `pow` and `std::pow` are all legal. In std's
`limb.cpls`: `bare.optional [fn, op, struct, enum, const, interface];`.
**A glyph operator is exempt from the marks** (owner: "`a`, because I
think doing `std::++` beats the purpose of making it with symbols"): it
can carry no path (`'a' geometry::<+> 'b'` does not exist), so it takes no
mark and a mark on it is refused, over (b) marking it, which could only
ever be `bare.only`. std's glyph operators (`++`, `==`, `<`) have no
`std::` form either; its word operators do (`std::plus`).
**A const is exempt too** (owner: "`b`"): `'pi'` alone is always a
variable, so another Limb's const is always written with its path
(`geometry::'pi'`) whatever its mark; a mark on a const, or `const` in a
kind list, is refused. Over (a) still requiring a mark, of which only
`bare.never` would mean anything.
**A Limb's own clashing name is reached through its root**, as any name
is: `my.list.append[…]` (design claude's reading; the owner: "yea"; the
root became `my.`, below). Open: how this sits with 2026-10-02's registry rule (a
published Limb writing another Limb's word operator bare is refused).

### 2026-10-06 — Without ASP, a literal wider than its type's bits is refused (owner: a; not built)

Signatar's literals are bit patterns (`let 'x': int8 = 128;` holds -128's
bits). Design claude asked about `let 'x': int8 = 256;` (256 needs 9 bits):
(a) refused, recommended, or (b) cut to the low 8 bits, `0`, as C does.
Asked twice about speed: neither costs anything at run time (both happen
while compiling; the program holds a plain 8-bit constant). Design claude
noted C compilers warn ("overflow in conversion changes value from 256 to
0") and Linux builds with `-Werror`. The owner: "`a`". So an `int8`
takes the literals 0 to 255, its bits read as signed. **The rule is
Signatar's** (owner: "Signatar's, then"), over IEG's: design claude
argued that literals and types are Signatar's, so one rule serves every
level, and a compile-time refusal is the kind of check the bottom layer
has (an ARM assembler refuses a constant it cannot encode). The owner
asked what C does (checked with Apple Clang: `int8_t x = 256;` builds as
0 with a default warning, as does `uint8_t`, and even `int8_t z = 128;`
warns "changes value from 128 to -128"), and whether Signatar has a
manual way (a literal is any pattern the writer picks, so the cut value
is written by hand; a run-time cut is assembly, IEG's raw calls or ASP's
`to.int8.bits[…]`).

### 2026-10-06 — std needs ASP: a manifest listing std without ASP is refused (owner)

Asked what `let 'x': uint8 = 200 plus 100;` does in a project with std but
no ASP ((a) wraps like the hardware, nothing checks, division by zero as
the chip does it; (b) std needs ASP; (c) wraps, but division by zero
always stops; design claude recommended c), the owner: "You genuinely
have to be shitting on my ass, if you installed std without fucking
ASP". So **std's maths always has true values and checks**: a
`CCA.cplr` that lists std must list ASP. Raw, hardware-like numbers are
for programs without std, on Signatar and IEG.

**std needs IEG too** (owner: "`a`"): std's operators on built-in types
are a power IEG grants (this morning's chain of permission), but the
oracle had let a manifest list std and ASP without IEG; design claude
asked (a) std needs IEG, refused without it, or (b) as built, IEG's grant
invisible underneath, after the owner's "I want the beginners to learn
it the diabolical way. Imagine if they didn't install IEG". The std
package already includes IEG.
**The std package includes ASP and Genny** (owner: "yea"): the packages
are base; base + Signatar; + IEG; **+ ASP + Genny + std**; all, with
source. Design claude pointed out that the 2026-10-06 package list
("base + Signatar + IEG + std") predated ASP (owner: "Fuck, I forgot ASP
even existed") and asked (a) ASP in the std package, recommended, or (b)
an ASP package of its own; and, since `cca new` lists ASP and Genny,
whether Genny goes in too. One "yea" to both.

### 2026-10-06 — A manifest lists std like any limb, by its registry ID and version; CCA uses the installed copy when it matches (owner: yea; not built)

Design claude asked how a `CCA.cplr` says "I want std", offering (a) a new
`cca::Dep[installed = "std"]`, recommended, (b) std's registry ID and
version like any limb, (c) a special `std = true`, having said std fits
neither `path` nor `id`. The owner asked who defined `std` (the name: the
owner, 2026-09-14, "`std` is one Limb"; the limb: not written yet, faked
in the oracle and CCA), then, pasting the two dependency forms: "Shouldn't
that work?" It does: the official std is published to the registry, so
`cca::Dep[id = "…", version = "1.0"]` names it; the installer's copy is
that same std, so CCA uses it when the ID and version match, and nothing
downloads; `cca new` writes the line. No new form, no special case. The
owner: "yea". Design claude's (a) was a form nobody needed: an ID says
*which* limb, not that it must be downloaded.

**std's registry ID** (owner: "do it yourself, I guess", after the
owner's own `openssl rand -hex 32` was masked in the terminal; design
claude ran `openssl rand -hex 32` once and took what came out):

```
fff4302dffc4a894a2bab6baf724eb21dfc05a2ad9818509d8001013d6761936
```

So a manifest lists std as `cca::Dep[id = "fff4302dffc4a894a2bab6baf724eb21dfc05a2ad9818509d8001013d6761936", version = "…"]`.

**std's version is the date and time of that release** (owner: "date and
time of update", after design claude asked for a first version, `0.1.0`
suggested; a version is any text up to 256 bytes, 2026-09-29), **in ISO
8601 with the releaser's own offset** (owner: "`b`, it does state tz,
right?"), as `2026-10-06T12:57:14+07:00`, over (a) UTC, `…Z`, which design claude
recommended because text sorts that way; with the offset stated, versions
compare correctly as times, only not as plain text. std's first version,
the one the oracle stands in for: `2026-10-06T12:57:14+07:00`, when this was recorded.

### 2026-10-06 — A file with no `CCA.cplr` gets what the machine's package installed (owner; not built)

Asked what `cca build hello.cpls` gives a file with no `CCA.cplr` beside
it ((a) Signatar only, (b) what `cca new` lists, (c) refused), the owner:
"Where did they get that CyborgPL?" Design claude: from one of the
website's packages, so **a lone file gets what that package installed**:
with the std package, std (as `cca new` would list it); with base +
Signatar + IEG, no std, so `plus` and `print` are refused. The owner:
"yea". The oracle stands in for the full package.

**The installer writes a machine-wide record of what it installed**
(design claude's proposal; the owner: "Oh, machine wide. Good idea"): one
per computer, beside no project, read by a lone file, by `cca new` to
know what to list, and rewritten by another codextra's installer to
override the default codextra. Design claude's sketch, not ruled: a
CyborgPL value like the manifest,
`cca::Installed[codextra = "Signatar", cores = [ … ], limbs = [ … ]];`,
somewhere like `~/.cyborgpl/`. **Its place is the one each OS gives for
that** (owner: "The one that an OS gives you for that"), over design
claude's `~/.cyborgpl/` and `~/.cca/`: on macOS `~/Library/Application
Support/CyborgPL/`, on Linux the XDG data folder (`~/.local/share/cyborgpl/`),
on Windows `%APPDATA%\CyborgPL\` (design claude's reading of each). Its
file name is **`installed.cplr`** (design claude's proposal; the owner:
"yea.").

**There is a CyborgPL uninstaller** (owner, same message: "And please,
there should be a CyborgPL uninstaller"). **With nothing named it removes
everything the installer put there** (base, codextra, cores, std,
`installed.cplr`); **with a piece named, only that piece**, rewriting
`installed.cplr` (so dropping std leaves Signatar and IEG); a user's
projects are never touched. Over (a) everything only and (b) pieces
only; the owner: "`c`". **The uninstaller comes with Synthia** (owner:
"Btw, the uninstaller comes with Synthia"; asked whether that means the
CyborgPL uninstaller is a Synthia command, or Synthia also gets one for
itself: "both"): CyborgPL is uninstalled through Synthia, and Synthia can
uninstall itself too. Spellings (`synthia uninstall`, `synthia uninstall
std`) are design claude's sketches. **Synthia only**: CCA has no uninstaller.
Design claude had asked about the twin ((a) `cca uninstall` doing the
work with `synthia uninstall` running it, as `tune`; (b) Synthia only,
every package including it; (c) Synthia only, a CCA-only user downloading
it); the owner: "Synthia only, just like how a CCA user would have to
change a limb's prefix themselves", so a CCA-only user removes CyborgPL
by hand. Whether a package includes Synthia: open.

### 2026-10-06 — The report says `parallel on N CPU cores` (owner: yea)

"Core" now names the category (IEG, ASP, …) and a CPU core. Design claude
found the CPU meaning in one user-facing place, the report's `parallel on
N cores` line (`design/diagnostics.md`, 2026-09-22), and none in syntax,
and proposed **`parallel on N CPU cores`** there, keeping "core" for the
category everywhere else. The owner: "yea". `CLAUDE.md`'s core split is
for the sessions and stays.

### 2026-10-06 — The unverified core/codextra warning's wording (owner: yea; design claude's wording)

Design claude's wording, in the `warning:` / `where:` shape of the errors:

```
warning: the core "Fastmath" (registry ID 7f3a…) is not verified
  where: CCA.cplr, cores
  note:  a core runs inside the compiler and can change what your
         program does. Use it only if you trust where it came from.
```

A codextra nobody has verified gets the same, with "codextra" for
"core". The owner: "yea". Still a warning, never a refusal (2026-10-05).

### 2026-10-06 — A codextra hands the base plain LLVM IR; compiler claude proposes the plugin interface (owner: yea)

Design claude asked what a codextra hands the base: LLVM IR, or a shared
middle form the base understands. The owner: "I mean, the base doesn't do
shit. So, idk". Design claude: then **plain LLVM IR**. The codextra does
all the thinking (parsing, types, borrows, Precog) and produces the IR; the
base hands it to LLVM, links and writes the program without reading it. A
shared middle form would make the base understand programs. Side effect:
RRx2 checks LLVM IR, so it can check any codextra's output, not only
Signatar's. The **plugin interface** for codextras and cores is
engineering inside CCA: **compiler claude proposes it** when it builds the
revision, bringing the owner only real choices. The owner: "yea".

### 2026-10-06 — Signatar gets kernel-grade raw memory, beside ownership (owner: e)

The owner: "Should Signatar have manual too? Too is the keyword here",
then, after design claude's options missed it, "the manual one that you
could use to make a kernel". Design claude: a kernel has no `malloc` to
borrow, so Signatar needs **pointers as ordinary values** (in a `let`, in
a struct; until now `ptr[T]` lived only in signatures, 2026-09-19),
**addresses as numbers and back** with pointer arithmetic, **volatile
access** the compiler never removes or reorders, and **raw memory with no
owner**, so a kernel writes its own allocator; owned, borrow-checked code
stays the default, raw memory the marked exception, as Rust's `unsafe`.
Over (a) Signatar's own `alloc`/`free`, (b) as before, (c) an allocation
core, (d) whole regions without the borrow rules. The owner: "`e`". It
reverses 2026-09-19's "pointers only in signatures", which was made for
calling C. **How it is marked** (owner: "since cores exist, does
Signatar really have to mark what's nocheck, and stuff?"): **Signatar
marks nothing**; it is raw and assembly-level throughout. The owner then
asked for "clear syntax for a kernel? But not a full safety wrap". Design
claude offered a separate markers core, ASP in pieces, or optional labels
in Signatar, and had called IEG "full power, no guard rails"; the owner:
"wasn't IEG fully unsafe decided, before codextra existed?" It was not:
what was recorded (2026-10-05 19:31) is "not so practical for normal
stuff", and IEG's raw operations carry contracts RRx2 verifies. The owner:
"I don't think IEG should be fully unsafe, but you did say that. IEG was
before codextras existed." So **IEG brings the markers**: `nocheck` and
`NO_FUCKING_CHECKS`, and the rule that raw memory and unproven code sit
under `nocheck`, beside its raw operations and `inline.LLVM`; **ASP brings
`check` and the run-time checks themselves**, which exist only once
numbers have true values. A kernel on Signatar + IEG gets clear syntax
without ASP's number safety (owner: "I think so. I mean, like Rust's
kernel mode also has clear syntax").

### 2026-10-06 — Signatar's numbers are raw, like the hardware; safe numbers come from their own official core (owner)

Asked whether there is another way than `-5` (std's `0 minus 5` or
`not.bit 4`, IEG's raw negate, assembly's `movn`), the owner: "Signatar
is supposed to be very lower-level, because it's not really supposed to
be use practically", so **Signatar has no negative literals**; with std,
`-5` is std's `-` applied to 5, written the same. Then: "should `let 'x':
int8 = -128;` really be possible with Signatar alone? ... shouldn't it
do it like hardware?" Design claude: hardware has bit patterns, so
`let 'x': int8 = 128;` would store -128's bits, and since Signatar's rules
reach every program, recommended keeping literals as checked values.
The owner: "There's cores, right?" Design claude: (a) Signatar raw, a
core brings the safe meaning, or (b) Signatar safe, a hardware core
opt-in, recommending (b). The owner: "You can't even write neg without
Assembly using Signatar, so that would be pretty weird", so (a); the
packages with std install the safe-numbers core too and the installer
lists it in new projects, as std. Asked which core: IEG, or **its own
official core**; the owner: "its own core. Pretty crazy how many things
you need to install to write a Hello World, but it's fine".

So: **Signatar's literals are bit patterns, as the machine's** (an
`int8` holding `128` holds -128's bits); **a separate official core makes
numbers values again** ("`200` is two hundred, refused in an `int8`",
the exact-and-checked-where-it-lands rules). A kernel takes IEG without
it and gets raw literals. **The core is ASP, "Abit Safer, Please"**
(owner), and **even with ASP, a way to write numbers like the hardware
stays, but never as the default** (owner: "There should still be a way to
write it like hardware, but shouldn't be the default"). Design claude
offered (a) per value, `to.int8.bits[128]`, recommended, or (b) a `raw { … }`
block; the owner asked where the `to.` family lives (not placed yet), and
then: "shouldn't Signatar be writing ASM for that?" So **Signatar has no
`to.` at all** (in Signatar alone, a reinterpretation is assembly, or
IEG's raw calls); **ASP brings the `to.` syntax**: checked conversions
(`to.int8['big']`) and the raw form `to.int8.bits[128]`; **std adds the
text and decimal conversions** (`to.int32["42"]`, `to.stored.str.utf8`)
into ASP's `to.`, through a power ASP grants it, as IEG grants std
`plus`. The owner: "seems good". **ASP carries the whole "numbers mean
their true values" set** (owner: "yea"): literals as values, "exact in
between, checked where it lands", the middle limit (the widest operand's
width, at least 64 bits), and the checked conversions. Open: exactly which rules ASP
carries (literals only, or all of "exact, checked where it lands" and
the middle limit).

### 2026-10-06 — Lists, maps and text: fixed arrays and text literals in Signatar; growing lists, maps and text operations in std (owner)

The owner: "The revision is the next thing that will be built. So, we
have to decide first", so the open items one at a time. First, by
whether a thing needs memory from somewhere (Signatar has no allocator):
**`array[T]`** (fixed size) and **text literals** (bytes in the program)
are Signatar's; **`list[T]`** (grows into a heap), **`map[K, V]`** (a heap
and hashing) and **text operations** (`str.bytes`, `++`, conversions) are
std's, which brings the allocator. `for 'x' in lend 'xs'` works on both;
std's list plugs into Signatar's `for` as `plus` plugs into its operator
syntax. The owner: "yea". A kernel makes its own lists, as Linux's
`list_head` and allocators are its own. So **plugging a type into `for`
is a power**, any limb can be granted (owner: "`a`"): IEG may grant it to
every limb, so a kernel's own list works with `for` as std's does, over
(b) an iteration interface, which would need Genny.

### 2026-10-06 — Code for one target: `on.target … { } else on.target … { } else { }`, chosen when compiling, and `on.machine` when running (owner: b; not built)

```
on.target aarch64 { nocheck inline.S { neg x0, x0 } with [ … ]; }
else on.target x86_64 { nocheck inline.S { neg rax } with [ … ]; }
else { let 'y' := -'x'; }          # plain code for everything else
```

The owner: "How do you write Assembly that runs on a specific target, and
runs on another in one file?" Not designed: `inline.S` assumed the one
target built for. Design claude showed C's `#ifdef` and Rust's
`#[cfg(target_arch)]`, and offered (a) the target on the block,
`inline.S.aarch64 { … }`, or (b) a compile-time choice around any code,
recommended (per-target needs reach past assembly, to a Windows-only C
function or a constant, and a fallback keeps a program portable; the
owner's 2026-10-02 Lilith idea, decided when compiling instead of
running). The owner: "`b`". Only the matching branch is compiled. Design
claude's reading: it is Signatar's, needing nothing at run time, so a
kernel has it. **Spelled `on_target`, one word** (owner: "If there's
nothing else after `on`, I think it should be `on_target`"): nothing else
is `on.`, so a dot would narrow nothing; the examples above were written
`on.target` before this. **It narrows** (owner: "`a`"): `on_target.chip
aarch64 { … }` (any OS), `on_target.os macos { … }` (any chip), and bare
`on_target aarch64-macos { … }` for one exact target. **Versions, both
ways** (owner: "what about OS versions? Chip version?", then, asked the
difference, "Both"): **when compiling**, `on_target` names a minimum
(`on_target.os macos 15 { … }`, `on_target.chip aarch64 sve { … }`), and
the program refuses to start on anything older, free at run time, as
Apple's deployment target or C's `-march`; **when running**, one build
carries both versions and checks the actual machine once at startup
(`on_machine.chip sve { … } else { … }`, design claude's sketch), as GCC's
function multiversioning, so it runs everywhere and is fast where it can
be. **`on.machine` is IEG's** (owner: "`a`"): it runs a check at startup
(`cpuid` on x86 anywhere; the OS's `sysctl` on an ARM Mac; a kernel reads
the chip's registers itself), and asking the chip what it can do is a raw
machine ability; over Signatar (which runs nothing by itself) and std
(kernels would lose it). **Back to the dot** (owner: "Wait a minute, you said `on_machine`, and
`on_target`. Shouldn't it use a dot now? Since, we now have two
narrowings of `on`"): by the owner's own rule, `on` now has two
narrowings, so **`on.target`** and **`on.machine`**, each narrowed again:
`on.target.chip`, `on.target.os`, `on.machine.chip`. The `on_target`
spelling above lasted the same morning. Open: `on.machine`'s layer (it
runs a check, so it may need IEG or the OS); `on` is a new keyword, so
synthia claude's name check needs it when built.

### 2026-10-06 — Generics are their own official core, not Signatar's (owner)

Design claude had placed the generics mechanism in Signatar and the
everyday interfaces (`Number`, `Ordered`) in std. The owner: "I don't
think generics should live in Signatar, tbh." Offered: (a) IEG, or (b)
their own official core, recommended (one core, one idea). Asked whether
kernels need generics (no, though many want them: Linux fakes them with
C macros, Rust for Linux uses them heavily) and whether they are
convenience with no trade-offs (none at run time; bigger programs, longer
builds and harder errors), the owner: "Ah, separate core". So **a
generics core** brings `<T>` announcements, bounds and `and`, `any`,
interfaces and `satisfies`; it requires only Signatar, so a kernel may
take it or not; std requires it (its `Number` and `Ordered` are generic);
the packages with std install it. **Lifetime groups (`@x`) stay in
Signatar**, being the borrow rules. **The core is Genny** (owner).

### 2026-10-06 — The one-operand operators go to std too: `not`, `not.bit`, `-'x'`; Signatar keeps `-5`'s sign (owner)

```
let 'y' := -'x';              # std's - (a prefix operator)
if not 'done' { … }           # std's not
let 'k' := -5;                # Signatar: a written number
let 'y' := negate['x'];       # Signatar + IEG, no std: IEG's raw call (sketch)
```

Design claude had recommended `and`/`or` to std and `not` staying in
Signatar (std cannot define prefix operators, 2026-09-20). The owner:
"why?", then "We could give more authority". So **a second power,
prefix operators**, defined by Signatar and granted by IEG to std (a big
power, changing what others' code means). Design claude then swung to
keeping the one-operand ones in Signatar on "Signatar is supposed to be
hella lower level, but not nothing"; the owner corrected: "I meant that
it's not nothing, but it's pretty lower-level (say writing fucking
Assembly for a lot of things). There's already IEG above, before std."
Shown the syntax at each level (std defining `op not ['a': bool]`, one
operand meaning prefix; a program with std; Signatar + IEG with raw
calls; Signatar alone with `inline.S`'s `neg`), the owner: "I think it's
like that." So `and`, `or`, `not`, `not.bit` and `-'x'` go to std; the
minus sign of a written number and `and.then`/`or.else` stay in Signatar;
IEG has the raw operations. **The bitwise operators (`and.bit`,
`or.bit`, `xor`, `shl`, `shr`) and `++` go to std too** (owner, later the
same day: "yea"), being two-operand. Spellings (`cca::Power.prefix_operators`,
IEG's `negate`, `bool_not`) are design claude's sketches.

### 2026-10-06 — Who decides permission: Signatar → cores → limbs → the program narrows (owner)

With the codextra added above cores, the owner: "First, we have to decide,
since codextra got added. Who decides permission?" Design claude
proposed a chain: **the codextra defines which powers exist and limits
what cores may do** (it "sets the rules cores play by"; no core may switch
off Signatar's borrow rules); **cores grant those powers to limbs**, to
every limb or to favourites by ID (07:01 this morning), only from the
codextra's list; **the program may take powers away, never add**. Over
(b) the codextra granting everything itself, and (c) cores granting
freely with the codextra out of it. The owner: "yea".

### 2026-10-06 — `for !c` is `for !cheat` (owner)

The owner: "Also, !c should be changed to !cheat." Design claude noted it
reads as what it means and ends the "not C" reading, and asked about the
twin: (a) `!c` refused naming `!cheat`, recommended, or (b) kept as a
short spelling. The owner: "`a`" (asked again in the evening, once MTM
existed for other spellings, whether `!c` should move there: "refuse").
In the oracle (the lexer reads
`!cheat` as one word, and still `!c`, so the parser can refuse it);
`examples/for_no_cheat.cpls`, reject tests `for_no_cheat_order`,
`no_cheat_old`; `parallelism.md`, `control-flow.md`, `style.md`, except
lines quoting a ruling's words.

### 2026-10-06 — The base goes as low as it can; a **codextra**, with more authority than a core, holds the language (owner; not built, being designed)

While deciding what the base keeps, group by group (comparisons to std;
`and.then`/`or.else` asked about: they skip their right side, which a std
operator cannot, though nested `if`s or inline assembly can do a given
case by hand), the owner: "Want to know how I want base should be? ...
Just low level as fuck, even if it then requires another category that
has more authority than a core." Design claude drew it out: the base
keeps only what nothing else can provide (reading files and modules,
loading cores and the new category and enforcing their grants, the bridge
to LLVM), and the new category gets what makes it CyborgPL (types, `let`,
`if`, loops, functions, the borrow rules, likely Precog), with more
authority than a core because it sets the rules cores play by. The owner:
"i think so, and maybe it would be called a seed, a codex, or a codextra
(cooler name for codex)". Design claude: "seed" clashes with check
claude's seeds, "codex" with OpenAI's coding agent; recommended
codextra. The owner: "codextra. Oh my god, I swear. When people say
"CyborgPL", they be meaning the official Codextra, like how they say BIOS
for UEFI".

So: **base (near-empty) → codextra (the language) → cores (IEG …) →
limbs (std …)**. **Codextra is a category** (owner: "`a`. Haha, mf, this
shit could make another Codextra be another lang, bro"), over there
being only one: anyone may write one, a whole rule set on the same base;
the official one is verified and others warn, as for cores. **The
official codextra is Signatar** (owner: "`Signatar`, signature but
cooler"). **Precog lives in Signatar, not the base** (owner: "`a`. Base
lang so low, even kernel would say wtf"): it proves things about
Signatar's own rules, so a codextra with other rules brings its own
checker or none. **Syntax is Signatar's too** (owner: "`a`"): a
codextra brings its own parser through its Rust plugin, so another
codextra may have wholly different syntax, and the
base does not know what a quoted name is. Following from both, design
claude's reading: what earlier rulings put in the base (SIMD, the minus
sign of a negative, `bool`, `and.then`/`or.else`) moves into Signatar.
The base keeps: reading files, loading codextras and cores, enforcing
grants, the bridge to LLVM. **Signatar installs itself** (owner:
"Shouldn't Signatar install itself? I mean, it's very powerful. Like,
say a Signatar installer from the website"), as the machine's default
codextra, which the base uses to read `CCA.cplr` (the chicken-and-egg:
the manifest is CyborgPL, CyborgPL is Signatar's). **The website offers
packages** (owner: "There will likely be 4-5 packages. Base lang, base +
Signatar, base + Signatar + IEG, base + Signatar + IEG + std, All
(include source too)"), replacing 2026-10-05's "the download is CCA and
IEG". **Another codextra's installer overrides the default** (owner:
"Your other codextra installer would just have to override it"), over
design claude's per-project first line (`#codextra X`, as Rust's
`rust-toolchain`). So one codextra is the machine's default at a time.
**Lilith is not a codextra and not in the CyborgPL family** (owner:
"Lilith does not fit, bro. That shit ain't in the CyborgPL family, it's
so different"), correcting design claude, which had floated it twice as
an example. **When this lands, the website says "Signatar CyborgPL"**
(owner: "we better say Signatar CyborgPL on the website, when it comes
in. Or else, it would just be an Assembly monster"), so the name names
the language people actually write, not the bare base. **When std came
in the package, the installer sets it up so `cca new` lists std** (owner:
"The installer auto adds std"), over design claude's (a), still empty;
2026-10-05's empty `CCA.cplr` stands for packages without std.
**Dialect or language** (owner: "Isn't a dialect a core, not a
codextra?", correcting design claude, then "I think so"): **cores make
dialects** (2026-10-05), still CyborgPL; **another codextra makes a
different language** on the CyborgPL base. "CyborgPL" means Signatar with
whatever cores. The base has no syntax and nothing to write in: it reads
files, loads the codextra and cores, enforces grants, and hands the
codextra's output to LLVM; it is "basically nothing" (owner), not a
backend like the JVM. What a codextra hands back (LLVM IR, or a shared
middle form) is open.
Shown the whole picture sketched (a project, std's `plus` on IEG's raw
`add`, IEG's grants, a kernel without std), the owner: "I think it's
good, because it's fun, we'll eventually need the no_std equivalent
anyways, and other stuff". Design claude's suggestion, recorded: let it
sit a little before compiler claude builds it, since it reshapes CCA.
 Open: what else Signatar holds and the base keeps; how it is installed; how it relates to Precog's and
RRx2's verification.

### 2026-10-06 — `CCA.cplr` is one real CyborgPL value, `cca::Manifest[ … ]`, not named `const`s (owner: c; not built yet)

```
cca::Manifest[
    name = "shop",
    prefix = "shop",
    cores = [cca::Core[name = "IEG"],],
    dependencies = [cca::Dep[path = "../std"],],
    deny = [cca::Power.inline_asm,],
];
```

Explaining the sketched `const 'deny' := …;`, design claude said the
names in `CCA.cplr` are setting keys CCA looks up (2026-09-29: "one plain
`const` per setting"), so in that one file a name does something; the
owner had noticed: "usually a name doesn't do stuff". Compared with
Synthia's and other package managers' keys (which mean something, in a
config format), the owner: "I think it would matter more. If `const
'build'` underneath is really a real `const`", then: "if the syntax looks
real, but it's not real. It feels like a child roleplaying their
language being hard". Proposed: (a) keep names as keys, CCA refusing
unknown ones (it does: "CCA.cplr has no setting 'dney'"); (b) one real
`const 'build' := cca::Build[ … ];`, one magic name left; (c) the file is
one value, no names, checked as real CyborgPL types, recommended (as
Zig's `build.zig` is real Zig). The owner asked what C does (C has no
manifest; Make, CMake, Meson, Conan and vcpkg each use a language of
their own), meaning option (c), then: "`c`. It may not look as complex,
but at least it's not much of a roleplay". So every `CCA.cplr` becomes
one `cca::Manifest` value: its settings are the struct's fields, typos
are the type checker's "no such field". Changes every existing
`CCA.cplr`, CCA's package manager and the oracle's manifest reader; to be
built when the owner says.

### 2026-10-05 — Two terms: a *core* extends the language, a *limb* is a library; the operators move out of the base (owner; not built, still being designed)

The owner: "I was thinking of making the base lang more lower-level ...
I wanted to move things like, plus, div, you know to std. And even more.
So, if we end up doing that, we better make limbs very capable." Design
claude laid out what that needs: machine operations in the base, the
right to define operators on the language's own types, precedence
(exists), and contracts Precog trusts; it suggested intrinsics. The
owner: "Isn't it supposed to be inline ASM for that?" Design claude: inline
assembly is one per chip and a black box to LLVM (no folding, no
vectorising, `nocheck` everywhere), so LLVM IR, portable assembly LLVM
optimises, `inline.LLVM { %r = add i32 %a, %b }`. The owner: "inline LLVM IR
would come with the Core limb. And the base language just uses lower
level stuff. So, right now, the limbs we got, from low to high: Core,
std. If there are problems, you could tell me." Design claude listed:
a Limb cannot add syntax; that makes Core privileged (against 2026-09-19's
"std is not a privileged one"); the oracle cannot run LLVM IR (keep its
built-in meaning, RRx2 checks Core's IR); Precog needs contracts; the
"checked where it lands" rules stay in the language. The owner: "What if
the Core limb really is just a separate CyborgPL you install?" Design
claude: workable, version-matched to CCA, with trust the issue: only a
verified core's contracts are believed. The owner: "That's what I meant.
Actually, let's have two lib-like terms here ... A `core` is a thing that
is kinda more of like a compiler mod, but a `limb` is a library?", then:
"It's a category, and I think if someone installs an unverified core,
just warn them, don't refuse. Maybe it's their own, maybe it's their
friend's. Reminds me of CUDA C++".

So far:
- **A core** extends what the language can do, like a compiler mod: it
  may use `inline.LLVM`, define operators for the language's own types,
  and give Precog contracts. Installed beside CCA, version-matched to it.
- **A limb** is a library, as before; std is a limb.
- **"Core" is a category**: several may exist. The official one is
  verified (RRx2 checks its IR against its contracts); an unverified core
  is used with a warning, never refused.
- **The official core is IEG, the Intense Energy Generator** (owner: "I
  was thinking that the IEG (Intense Energy Generator) core should not be
  so practical for normal stuff"): raw and low-level, so `plus` and the
  others go up to std (design claude's reading of "not so practical").
- **A core need not put its name in the syntax** (owner: "Since a core has
  so much authority, it isn't necessarily forced to label its name in
  syntax. So, it really does depend on the core"): no `ieg::` unless the
  core wants one.
- **A limb may be granted powers, fewer than a core's** (owner: "could we
  grant a limb a lot of authority, but not as much as a core, of
  course"). Which: "Just what's necessary", so one power for now:
  defining operators on the language's own types, which std needs for
  `plus` on `int32`. A limb never writes `inline.LLVM` and its code is
  still proven by Precog; only a core is believed on its word.
- **A power is declared in the limb's manifest** (owner: "In there seems
  good"): `const 'powers' := [cca::Power.operators_on_builtin_types,];` in
  `CCA.cplr`, and the same in `Synthia.ttsr`'s syntax; a build using the
  limb shows it. Spellings are design claude's, not yet ruled.
- **An easier way than writing LLVM IR** (owner: "there should be an
  easier alt to writing LLVM shit, maybe in the std limb, or in a higher
  level core"): (a) std gives the useful LLVM operations friendly names,
  built on IEG (`std::bits.count_ones`, `std::bits.leading_zeros`,
  `std::bits.swap_bytes`, `std::math.fused_multiply_add`), recommended;
  (b) a second, higher core naming every LLVM operation one-to-one. The
  owner: "`a`". Names are design claude's examples, not ruled.
- **The download is CCA and the IEG core, nothing more; std is installed
  like any limb** (owner: "the highest shit you can get from the website
  is, by installing the language is a core", after the joke thread "it's
  in the std limb" / "It doesn't"). Design claude had recommended std
  shipping with CCA. And **`cca new` writes an empty `CCA.cplr`**, std not
  listed: design claude offered (a) std listed by default, recommended,
  or (b) empty; the owner: "`b`. I mean, lol. I don't think you're a
  beginner, if you sign up for Precog". Today's oracle and CCA still
  reach `std::` without a listing; that changes when this is built.
- **std stays in CyborgPL** (2026-09-30), **and may call the Rust runtime
  for what CyborgPL cannot express yet**. The owner first: "I think that
  the std limb could be in Rust first, just, just, just in case". Design
  claude: that reverses 2026-09-30, puts std outside Precog (every call a
  black box, or std trusted like a core), cannot give an inlinable `plus`,
  and means writing std twice; the `no_std` Rust runtime is already the
  safety net. The owner: "Your suggestion is good, honestly".
- **A core is a Rust plugin for CCA plus CyborgPL code** (owner, on the
  plugin's risk: "if they already had malware, they're done for anyways.
  And I mean, doesn't inline.LLVM come with IEG?"). The plugin extends CCA
  (new syntax, checks, names without a prefix); IEG's plugin is what adds
  `inline.LLVM`. Design claude had proposed cores in CyborgPL with powers
  from a fixed list CCA offers, no code of their own in the compiler; the
  owner's point stands that installing a core is already trusting it.
  Design claude's remaining note: CCA needs a stable plugin interface, so
  its insides can change without breaking every core.
- **Cores can depend on cores** (owner: "I mean, cores can depend on
  cores"), so a GPU core can build on IEG's `inline.LLVM`.
- **Cores may make dialects of CyborgPL, and that is fine** (owner:
  "cores are pretty powerful, so powerful that I think they could create
  separate dialects of CyborgPL. But, that's honestly fine").
- **A limb says which cores it is built for** (owner: "Oh, yeah. Limbs
  should say what cores they are built for"), in its manifest, so a file
  using a core's syntax is never a surprise. Design claude's suggestion,
  after the owner allowed dialects. **In a list of its own**, beside the
  dependencies (owner: "a", over one list with a marker):
  `const 'cores' := [cca::Core[name = "IEG"],];` in `CCA.cplr`, a
  `[my cores]` section in `Synthia.ttsr` (spellings design claude's).
- **Where the speed lives** (owner: "should the crazy perf behavior be
  with IEG, or the base lang? ... the auto para, and stuff"). Design
  claude first said the base; the owner: "Imagine you have don't even
  fucking have plus, and then a loop somehow auto para in your OS kernel,
  lol", which is right: running across cores needs threads, a runtime, an
  OS. So: **Precog's proofs and SIMD stay in the base** (they need
  nothing); **splitting loops across cores goes to a higher-level core
  that requires IEG** (owner: "I'll say a higher level core that requires
  IEG"), over (a) IEG with an off switch and (c) std with a second power
  (the owner: "idk if a limb has that much authority"). A program without
  it, a kernel, runs its loops in order, still proven and still SIMD, and
  `for para` is refused there. **GPU offload is a core of its own**, also
  requiring IEG, and `inline.MSL` moves into it, being syntax (owner:
  "`b`", over one "speed" core for both; design claude's reason: many
  programs want threads and no GPU, and Metal is not everywhere). Open:
  both cores' names. **They require IEG, not std** (owner: "Nah, it's
  fine. They already have base, so yeah"): a program with the parallel
  core and no std has threads but writes its own `plus` from IEG's raw
  operations, which the owner found funny ("I have to make my own plus
  operator, and then my loop somehow auto para") and accepted. Design
  claude had offered (a) both require std too, recommended, (b) `plus`
  back into IEG, (c) leave it. **`concur` joins the parallel core**
  (owner, 2026-10-06: "`a`"): tasks, channels, `whichever`, timers and
  `concur.detached`, beside the automatic parallel loops, sharing one
  thread pool, over a core of its own. **The parallel core is MCPS, "More
  Concurrently Parallel Shit"** (owner). **GPU: one core for automatic
  offload on every GPU, plus three vendor cores for hand-written GPU code**
  (owner: "one GPU core that adds support for all without manual, and 3
  more for manual too"): Apple's (`inline.MSL`), AMD's and NVIDIA's, each
  bringing its own inline language; the automatic one translates plain
  `for` loops for whatever GPU the target has (2026-09-21's automatic
  offload). Design claude had first proposed one core with backends,
  warning a vendor core would lock a program to one GPU; the owner: "Isn't
  that like saying "Why does Valorant not work on Linux?"", then that
  listing all of them runs on all. **Names: AutoGPU** for the automatic
  core (owner), and for the hand-written ones **Metal**, **Nvidia** and
  **Vulkan** (owner: "We could just say "Vulkan", "Nvidia", and "Metal"").
  Design claude had proposed MetalGPU / CUDAGPU / HIPGPU and cautioned that
  Metal, CUDA and ROCm are their companies' trademarks; the owner: "we
  aren't trying to trademark our limb, and core names. Except the
  categories, and stuff (like Signatar)". The third core is Vulkan rather
  than an AMD-only one: Vulkan runs on AMD, NVIDIA, Intel and Android
  alike.
- **What the base keeps, decided one group at a time** (owner: "let's
  decide number 1 first"). **Comparisons go to std** (`==`, `!==`, `<`,
  `>`, `</==`, `>/==`), as `plus` does (owner: "`a`"); the base keeps
  `bool` and `true`/`false`, so `if` has something to test. Precog's
  guard proofs survive: std's `<` is built on IEG's compare, whose
  contract Precog sees through. Kept in the base so far: syntax, types,
  `let`, borrows, Precog, SIMD, the minus sign of a negative and `-'x'`
  (design claude's recommendation, not ruled), `and.then`/`or.else`
  (they skip their right side, which a std operator cannot).
- **A core may give a specific limb more authority than others** (owner,
  same day: "I think that a core could give a specific limb more
  authority than others"). Design claude offered (a) only the core says
  it or (b) the limb asks and a core grants; the owner wanted both tiers
  ("depends on how much authority"), then settled it: "A core should be
  able to say the permission of a limb. Say, this core gives any limb this
  ability, and its favourite limb extra ability". So **a core grants
  powers to every limb built with it, and extra powers to favourites,
  named by their registry ID** (design claude's point: names repeat, IDs
  do not). A limb still lists the powers it uses, visible in every build,
  and each works only if a core it is built for granted it. Then, asked
  for better ideas, design claude added two, and the owner: "Yea":
  **the program's own manifest may take powers away, never add** (as
  Deno's and phones' permissions: the one running decides last), and **an
  upgrade that starts using a new power is flagged** before it goes ahead.
  Spellings (`grants_to_all`, `grants`, `deny`, `cca::Power…`) are design
  claude's sketches.
- Open: what the base keeps exactly, how a core is installed and named in a
  manifest, what the unverified-core warning says.

### 2026-10-03 — A shift on an exact answer works at its typed operands' width (owner)

```
let 'v': uint32 = 3;
let 'b': uint32 = ('v' mod.floor 16) shl 2;    # 12, shifted as 32 bits
('v' plus 1) shl 30                            # refused: 2^32 does not fit 32 bits
```

CCA's open question since 2026-09-23, raised again by compiler claude
and check claude once division became exact (15c5c71): which width a
shift works at when its left side is an exact answer. The 2026-09-23
ruling ("any operand whose type it can work out is enough", with this
very example) and the oracle already answer it: the width of the typed
operands inside, as `plus.wrap` takes it. Design claude asked the owner
to confirm, explained twice ("What"); the owner: "Yeah". The `widths`
example's question is settled; CCA builds it.

### 2026-10-03 — Hardware-style division: `div.trunc.wrap` and the other three; no `mod.X.wrap` (owner)

```
let 'a': int8 = -128;
'a' div.trunc.wrap -1     # -128: wraps, as ARM gives it
'a' div.trunc -1          # 128: the true answer, checked where it lands
```

From the "might add later" list ("hardware-style division"); the owner:
"number 4 is wanted". With the tilde gone, the spelling is the owner's
own `div.trunc.wrap`. Asked which divisions get it: (a) all four, since
`.wrap` means "wrap instead of overflow" on every operator,
recommended, or (b) only `div.trunc`, the hardware's own; the owner:
"`a`". The twin, `mod.X.wrap`: `mod` never overflows (the smallest
value `mod` -1 is 0), so refusing it was recommended; the owner asked
what `mod.floor` is for (the remainder of `div.floor`, the clock-safe
one) and how to do the hardware's `mod` (`mod.trunc`, C's `%`, which
needs no wrap; x86 alone faults there), then: "Ah, okay then". So
`div.floor.wrap`, `div.ceil.wrap`, `div.trunc.wrap` and `div.near.wrap`
give the true quotient wrapped at the type's width (a typed side is
needed, as for `plus.wrap`); `mod.X.wrap` is refused. In the oracle;
`examples/div_wrap.cpls`, reject test `mod_wrap`.

### 2026-10-03 — Integers of any width, marked `!std_`: `!std_int24`, `!std_u256` (owner; built)

```
let 'sample': !std_int24 = -8_000_000;    # our style
let 'sample': !std_i24 = -8_000_000;      # Rust's alias
let 'flags': !std_uint4 = 9;              # !std_u4
let 'hash': !std_uint256 = …;             # !std_u256
```

The owner asked which types LLVM has that CyborgPL lacks. Design claude
listed integers of any width (LLVM's `iN`), scalable vectors and matrix
tiles as worth a look, `fp128` as ruled out (2026-09-23), and `x86_fp80`,
`ppc_fp128` and LLVM's internal types as not worth it, recommending the
first; from memory, not LLVM 23's manual. The owner: "1 seems like a good
one". Widths: (a) any, (b) whole bytes, recommended, (c) a short list;
the owner: "`a`. With `!std_` in front. It means not standard. And of
course, do our style, and Rust alias". So any width LLVM takes, with the
`!std_` mark: `!std_intN` / `!std_uintN` and Rust's `!std_iN` / `!std_uN`;
`int8` to `int128` stay plain. `!std_int32` (the mark on a standard
width) is refused, naming `int32` (owner: "Refuse"). The top: (a)
LLVM's own limit, about 8 million bits, or (b) a cap of 4,096,
recommended; the owner: "`a`, I mean there's already a disclaimer in
front". Built up to 128 bits the same day (the owner: "Yeah", after
seeing Rust's alias spelled out; design claude noted Rust itself has
none past 128); above 128 the oracle needs bigger numbers than it holds,
so `!std_uint256` is refused as not built. Then the owner: "Wait, what
about more than 128?", and to build it now, writing the big numbers
into the oracle rather than adding a library: "Do it now, `a`". That
met the 2026-09-23 rule's middle limit ("the one limit is the machine's
64 bits"), which would refuse nearly every `!std_uint256` sum. Proposed:
(a) the limit is the widest operand's type, at least 64 bits, recommended
(it also lets two large `int128`s add, which the 64-bit limit refused);
(b) 64 bits always. The owner asked what that meant, then the
performance difference (none for code of 64 bits or less; a 256-bit add
is four instructions, the cost of the type itself), then: "Ah, okay". So
in between, the limit is the widest operand's width, at least 64 bits.
LLVM's own top width is 8,388,608 bits (2^23, checked with `llvm-as`).
Built the same day: the oracle's own big numbers (`bigint.rs`: add,
subtract, multiply, divide, power, compare, wrap, print and parse,
tested against i128 and Python), used whenever a value or a result is
past i128; literals past i128; ranges and conversions at any width; the
middle limit as ruled; wrapping at 127 bits and up on big numbers, which
also fixed `uint128` wrapping (it had wrapped at 2^127). Then bitwise
operators, shifts and `not.bit` on big values (the owner: "Do the bit
shit"), at the type's true width, which also let `uint128` shift by 127
and `not.bit` flip all 128 bits; `examples/wide_bits.cpls` (checked
against Python), reject test `wide_shl_overflow`. SIMD lanes: (a) only
standard widths (8, 16, 32, 64), recommended, since no chip has a 4-bit
or 256-bit lane and an emulated vector is slower than plain code; (b)
any width filling 128 bits. The owner: "`a`". `simd[!std_uint4, 32]` is
refused; reject test `simd_std_lane`.
`examples/wide_integers.cpls` (checked against Python), reject tests
`wide_past_in_between`, `wide_does_not_fit`; `std_too_wide` now past
LLVM's limit. `examples/any_width.cpls`,
reject tests `std_on_standard`, `std_too_wide`, `std_not_a_type`,
`std_width_overflow`, `types.md`.

### 2026-10-03 — A Limb's own narrowed operators and operator paths are built (owner)

The owner: "what about the writer being able to make a `std::pow.` op?"
Design claude first read it as adding to std's own names and advised
against it (only std writes `std::…`); the owner: "I meant a limb being
able to make their own op like that", which 2026-10-02 had decided
(`pub op pow.wrap […]`, a path on a word operator) but the oracle could
not read. Asked to build it: "yea". So `pub op mean.low […]` in a Limb,
used `'x' mean.low 'y'` or `'x' areas::mean.low 'y'`; a path picks only
that Limb's operator (refused, naming the Limb, where it has none); a
dotted operator may sit in a `precedence` level. In the oracle;
`examples/limbs/areas` and `examples/uses_a_limb`, reject test
`op_path_wrong_limb`. The registry's refusal of the bare form is the
registry's, not the language's, so nothing checks it here.

### 2026-10-03 — A Limb's operator precedence: `precedence.low_first` / `precedence.high_first`, `level.when_chained.…` (owner; not built yet)

```
precedence.low_first {          # level 1 binds loosest (good practice)
    level 1 { ==  ~= }          # a language operator pins its level: ~= binds like ==
    level 2 { plus }
    level 3 { times  dot }      # dot binds like times
    level 4 { scale }           # above times, by number
}
precedence.high_first { … }     # the same, level 1 binding tightest
```

Design claude proposed `levels { 1 > times; }` with `pub op.1 pow`; the
owner: "precedence { level 1 { x } level 2 { xx } … } kinda like that".
Design claude read it as: a level lists operators; a language operator in
it pins the level to where that operator is; the Limb's own operators sit
by number; an operator in no level keeps brackets; the definitions carry
no number. Asked "any better ideas?", design claude offered one line per
level, loosest first, no numbers (`precedence { == ~=; plus; times dot;
scale; }`); the owner: "Seems a bit off for a language with
`to.useful.str.utf8` just to convert", so the numbered blocks stay. The
owner: "There should be two versions of `precedence` using a dot narrow.
Like, say highest priority to lowest". Names offered: (a) `high_first` /
`low_first`, recommended, (b) `tight_first` / `loose_first`, (c)
`descending` / `ascending`; asked whether bare `precedence` is refused.
The owner: "`a`, bare refused, low_first is good practice". Operators of
two Limbs landing in the same gap need brackets between them (design
claude's, from 2026-10-02's "b"). Then the order of a chain (`2 pow 3
pow 2` takes the right pair first, as in maths and the old `xx`):
design claude offered `level 5.right { pow }`, plain `level` going left;
the owner: "yes, but bare is refused. Shouldn't it be `level.right 5 {
pow }`?" So every level says `level.left N` or `level.right N`, and a
level holding a language operator must agree with that operator's
direction (`level.right 2 { plus }` is refused; the owner: "yea").
Comparisons never chain (`1 < 2 < 3`, 2026-09-14), so design claude
proposed a third, `level.none`, required for a level holding `==`; the
owner: "yea. Though, I think it should be
`level.when_chained.right_first`". The third in that pattern: (a)
`level.when_chained.refused`, recommended, (b) `level.never_chained`;
the owner: "`a`", and asked `refused` or `refuse` (`refused`: each last
word says what happens, as `left_first` does; "yea"). So:

```
precedence.low_first {
    level.when_chained.refused 1 { ==  ~= }
    level.when_chained.left_first 2 { plus }
    level.when_chained.left_first 3 { times  dot }
    level.when_chained.right_first 4 { pow }
}
```

The `level.left` / `level.right` spellings above were replaced the same
day. Last, a negative beside std's `pow` (the old `-2 xx 2` was -4):
design claude asked whether a level may hold the negative under a name,
(a), or a negative beside a Limb's operator always needs brackets, (b),
recommended (`-2 pow 2` is easily misread); the owner: "Isn't it
`neg`?" (it is, from `op neg`), then "`a`". So a level may hold `neg`,
pinning it where a negative sits, and std's `pow` can sit above it:

```
level.when_chained.refused 4 { neg }
level.when_chained.right_first 5 { pow }    # -2 pow 2 is -(2 pow 2) = -4, as xx was
```

Built in the oracle the same day (the owner: "go ahead"): the language's
levels in hundreds so a Limb's fit between them, a level with no language
operator placed by order between its neighbours; std's `pow` above `neg`,
right pair first, so `-2 pow 2` is -4 and `2 pow 3 pow 2` 512 again (the
reject test `pow_beside_negative` removed). `examples/precedence_levels`;
reject tests `precedence_bare`, `level_bare`, `level_wrong_chain`,
`level_bit_op`, `level_refused_chain`, `level_two_numbers`. Oracle limits:
the block applies to the file it is in; two Limbs in one gap unchecked.
(2026-10-05: the first limit is gone. The oracle reads every file's
`precedence` blocks, dependency Limbs' too, before parsing the program,
so `areas`' levels for `mean` apply in `uses_a_limb`; two Limbs giving
one operator different levels is refused.)

### 2026-10-03 — The operator words, settled for building: `.wrap`, `op plus`, the symbols refused, `pow` with brackets (owner)

```
'a' times 'b'     'a' plus 'b'     'a' minus 'b'     -'a'   # a negative keeps the symbol
'h' times.wrap 31     'n' plus.wrap 1     'n' minus.wrap 1  # were x~ +~ -~
('a' pow 'b') plus 1      'h' pow.wrap 3                    # std's; were xx, xx~
op plus ['a': Vec, 'b': Vec] --> Vec { … }                  # a type's own `plus`
fold[lend 'xs', 0, op plus]                                 # an operator as a value
```

The owner asked to build the decided `times`/`plus`/`minus` first ("It's
1-3"), and to ask before building each twin ("Ask first"):
- A user type's `op + […]`: asked whether it makes a type or says what
  the language's `+` does for one (the latter; `struct` makes the type);
  becomes `op plus […]` ("yea").
- The wrapping forms: (a) leave `+~ -~ x~` for now, recommended, or (b)
  refuse them; the owner: "design". Proposed (a) `plus~`, (b)
  `plus.wrap`, recommended (it matches `div.floor` and the owner's
  `div.trunc.wrap`); the owner: "`b`". `xx~` becomes std's `pow.wrap`
  ("yea").
- The old spellings `x`, `+`, two-number `-`, `xx` and the tilde forms:
  (a) refused naming the word, recommended, or (b) kept as other
  spellings; the owner asked what binary `-` is (subtraction, two
  numbers; a negative keeps `-`), then: "`a`".
- `op +` as a value becomes `op plus`, `op times`, `op minus`; `op neg`
  stays ("yea").
- `pow` now with brackets, as a Limb's operator has no level until the
  `levels` question is settled (a, recommended), or keep `xx` until then
  (b); the owner: "`a`".

Built in the oracle the same day: every example, reject test and the code
in the design notes rewritten (a script, then lines it damaged fixed by
hand: the custom-operator character list, a diagram's `+`); `-2 pow 2`
is refused, a negative being an operator beside a bracket-less one;
`examples/word_operators.cpls`; reject tests `plus_old`, `times_old`,
`pow_old`, `wrap_old`, `op_value_old`, `op_decl_old`,
`pow_beside_negative`. Not yet told to compiler, check or synthia claude
(no messages, to save usage); no keyword was added.

### 2026-10-03 — The long arrows are the main spelling: `-->` and `<--`; `->` and `<-` are the others (owner)

The owner, after `-->` and `<--` went in as aliases: "Long arrow is main
in CyborgPL". Design claude asked whether `<--` too, and whether to
rewrite everything now (recommended yes to both); the owner: "yea, both".
Every example and reject test now writes `-->` / `<--` (189 files);
`examples/short_arrow.cpls` keeps the short ones working. The design
notes are rewritten too, except lines holding a quotation, so the
owner's words stay exact (a few sample lines with a string in them keep
the short arrow, still valid). The oracle's own error messages still
print `->`. Entries in this log keep the spelling of their day. Not yet
told to compiler or check claude (no messages, to save usage).

### 2026-10-03 — `-->` is another spelling of `->`, and `<--` of `<-` (owner)

From the "might add later" list. The owner: "Implement the `-->` alias,
that's easy." It goes wherever `->` does (a return type, a function
type, an asm or `inline.MSL` binding). A side effect, noted by design
claude: `-->` can no longer name a custom operator (a run of operator
characters was one, 2026-09-20). In the oracle's lexer;
`examples/long_arrow.cpls`, `functions.md`. Then, asked whether `<-`
gets `<--` too (design claude had not asked before building `-->`): "yes,
next time, ask". So `<--` is another spelling of `<-` (asm bindings,
`print.weird`), and `<--` no longer names a custom operator either. Not
yet told to compiler or check claude (the owner asked for no messages,
to save usage).

### 2026-10-02 — A Limb's word operator may carry its path and narrow with `.`; the registry refuses the bare one (owner; not built yet)

```
'a' pow 'b'             # legal
'a' std::pow 'b'        # legal: the path, as any name from another Limb
'a' std::pow.wrap 'b'   # a narrowed form, which std declares: `pub op pow.wrap […]`
```

Told that a Limb's `pub op` is used bare (2026-09-23), the owner: "There
should be a way to make a limb word op, with it being able to do `std::`,
and `.` narrows". Design claude asked whether the path is (a) required,
or (b) allowed beside the bare form, recommending b (maths reads better
bare; the path settles two Limbs defining one word for the same types,
an error until now; glyph operators like `~=` stay bare). The owner:
"`pow` & `std::pow` are legal, BUT the official limb registry might not
accept you". Design claude read it as the registry refusing a published
Limb that writes another Limb's word operator bare, as `#strict` is good
practice for public Limbs, and asked refuse or warn. The owner: "Always
refuse, unless I override". So: both legal in the language; the official
registry refuses a published Limb using a bare word operator from another
Limb, unless the owner overrides it for that Limb. Glyph operators are
not covered by the path. Open: whether a Limb's operator gets a
precedence (asked).

### 2026-10-02 — `times`, `plus`, `minus`; `xx` leaves the language for std's `pow` (owner; not built yet)

```
'a' times 'b'      # was 'a' x 'b'
'a' plus 'b'       # was 'a' + 'b'
'a' minus 'b'      # was 'a' - 'b'; a negative stays `-5`, `-'x'`
'a' pow 'b'        # was 'a' xx 'b': std's operator now, no longer the language's
```

From hardware-style division: design claude offered `div.trunc~`, after
`+~`; the owner: "Isn't it `div.trunc.wrap`?" (nothing is spelled `.wrap`
yet; wrapping is the tilde, 2026-09-20), then: "we might just change our
ops to words". Design claude laid out what is already words and what is
still a symbol, and recommended only the wrapping ones becoming words,
`+ - < ==` being known to everyone. The owner: "Don't implement yet. `x`
becomes `times`, `xx` becomes `std::pow` (there should be a way for a
limb to make a word op, without it becoming `[]`), and gets removed from
base. `+` becomes `plus`, `-` becomes `minus` (only for `5` `-` `5`)".

A Limb making a word operator used without `[]` exists already: `op`
(2026-09-20) and a `pub op` used bare by every Limb that depends on it
(2026-09-23). Open, asked one at a time: how std's power is written at
the use (`'a' pow 'b'`, by that rule) and its precedence (a Limb's
operator has none: parentheses beside any other); the wrapping forms
(`+~ -~ x~ xx~`, and `div.trunc~` or `.wrap`); `op +` / `op x` as values;
the comparisons and `++`, unchanged unless the owner says.

### 2026-10-02 — Division is exact and checked where it lands, as `+ - x xx` are (owner: a)

```
let 'a': int8 = -128;
std::print.useful.stdout[('a' div.trunc -1), \n];   # 128: a print takes the true answer
let 'b': int8 = 'a' div.trunc -1;                    # refused: 128 does not fit int8
```

Compiler claude found that CCA refused `-128 div.trunc -1` on `int8`s
inside a print, working at the operands' type, while the oracle printed
128. The 2026-09-23 rule ("plain math is exact; it is checked where it
lands") named only `+ - x xx`. Design claude asked whether division
follows it: (a) yes, recommended, one rule for all maths; (b) checked at
the operands' type. The owner asked the performance difference (none:
both need one compare for a divisor of -1 where Precog cannot rule it
out) and what fits the hardware more ((b): chips divide at the operands'
size; ARM silently gives back the smallest value, x86 faults, C calls it
undefined; but hardware wraps `+ - x` too, and the owner chose the true
answer there). The owner: "`a`". The oracle already did this;
`examples/div_lands.cpls`, reject test `div_lands_too_small`,
`operators.md`. CCA changes.

### 2026-10-02 — Every division says how it rounds: `div.floor`, `div.ceil`, `div.trunc`, `div.near`; `mod` too (owner)

```
7 div.floor 2    # 3      -7 div.floor 2    # -4
7 div.ceil 2     # 4      -7 div.ceil 2     # -3
7 div.trunc 2    # 3      -7 div.trunc 2    # -3
7 div.near 2     # 4      5 div.near 2      # 2 (a tie, to even)
7.5 div.near 2.0     # 3.75: ordinary division on a bin or deci
7.5 div.floor 2.0    # 3.0, and 7.5 mod.floor 2.0 is 1.5
```

After `div.trunc` moved into the language, design claude listed what else
could take a `.` (`div.ceil`, saturating maths) and the owner asked what
bare `div` does (floors on integers, ordinary division on bins and
decis). The owner: "Make bare `div` illegal". Asked whether on integers
only (a, recommended: only there is there a rounding to choose) or every
number (b): "`b`". The ordinary division on bins and decis then needed a
name: (a) `div.near`, naming its rounding as the others do, recommended,
(b) `div.real`, (c) `div.frac`: "`a`". Whether every rounding works on
every family (a, recommended: `7.5 div.floor 2.0` is Python's `//`,
`7 div.near 2` the nearest whole, a tie to even) or only the natural
ones (b): "`a`". Whether `mod` must say it too, one for one, each the
remainder its `div` leaves, `a == (a div.X b) x b + (a mod.X b)` (b,
recommended) or stays bare as the floor one (a): "`b`". Design claude
then found its own slip: on a bin or deci `div.near` leaves nothing over,
so `mod.near` there is always 0 or a rounding crumb, not IEEE's
`remainder()` as it had said; it proposed refusing `mod.near` there. The
owner: "refuse, if you're sure". So:

- `div.floor`, `div.ceil`, `div.trunc`, `div.near` and the four `mod.X`
  on integers, bins, `brain16` and decis; bare `div` / `mod` (and
  `op div` / `op mod`) refused naming the four; `div.round` refused.
- On integers every quotient is whole. On bins, `brain16` and decis,
  `div.near` is ordinary division rounded to the type, `floor`/`ceil`/
  `trunc` a whole quotient, and `mod.near` is refused.
- `mod.floor` is the old `mod`: the divisor's sign, and a result that
  rounds up to the divisor is `0` (both earlier today).
- Precedence and the parenthesis rules are `div`'s and `mod`'s.

In the oracle, checked against Python (`//`, `%`, `math.ceil`,
`math.trunc`, `round`, `math.fmod`); every example and reject test
rewritten (an integer `div` became `div.floor`, a bin or deci `div`
`div.near`, every `mod` `mod.floor`); `examples/div_rounding.cpls`; reject
tests `div_bare`, `mod_bare`, `div_unknown_rounding`, `mod_near_fraction`,
`op_div_bare`; `operators.md` and the samples in other notes.

### 2026-10-02 — `div.trunc` and `mod.trunc` are operators in the language, not std's (owner: a)

```
-7 div.trunc 2        # -3   (was std::div.trunc[-7, 2])
-7 mod.trunc 2        # -1
'angle' mod.trunc 360.0   # C's fmod
```

The owner asked whether `div.trunc` is in std or the base language (std's,
2026-09-20), then: "I think `div.trunc` should be in base". Design claude
offered (a) an operator, narrowing `div` as `.bit` narrows `and`,
recommended, or (b) a built-in function, `div.trunc[a, b]`, and asked
whether `mod.trunc` moves with it. The owner: "`a`, and yea". Same
precedence as `div`; `mod.trunc` beside another operator needs
parentheses, as `mod` does. `div.trunc` takes integers, `mod.trunc`
integers, bins and `brain16` (today's earlier ruling). `std::div.trunc`
and `std::mod.trunc` are refused naming the operator. In the oracle;
`examples/division.cpls`, `examples/mod_bin.cpls`; reject tests
`div_trunc_std_moved`, `mod_trunc_beside`; `operators.md`.

### 2026-10-02 — `mod` on a bin or `brain16` takes the divisor's sign, as on integers and decis (owner: a)

```
-370.0 mod 360.0              # 350   (was -10 in the oracle)
7.5 mod -2.0                  # -0.5
std::mod.trunc[-370.0, 360.0] # -10, C's fmod, by name
```

Compiler claude's question, from check claude's brain16 batch: integers
(2026-09-20) and decis take the divisor's sign, bins were never decided,
and the oracle gave the dividend's sign by accident (Rust's `%`, not
listed as a provisional pick). Proposed: (a) the divisor's sign; (b) the
dividend's, as C's `fmod`. Compiler claude and design claude recommended
a: one `mod` in every family, an angle stays 0 to 360, and C's stays
reachable through std. The owner asked the performance difference (both
start with the same `fmod` routine, no chip has one instruction for it;
a adds a compare and an add, none where Precog proves both signs), then:
"`a`". In the oracle, with `std::mod.trunc` taking bins; checked
against Python's `%` and `math.fmod`. `examples/mod_bin.cpls`,
`operators.md`. Then the edge: a tiny negative over a large divisor
rounds up to the divisor (`-1e-20 mod 360.0` is `360.0` in Python).
Design claude offered (a) leave it, as Python, or (b) make it `0`, so the
answer is always below the divisor, recommended; the cost is one compare
and select after the fix-up. The owner: "`b`".

### 2026-10-02 — Lines in an `extern.C` block may each be `pub`; a `pub` on the block overrides (owner)

Design claude had refused `pub` on a line inside a block (its own pick).
The owner: "Why is `pub` per line illegal?" Design claude: no strong
reason; it stops one block mixing public and private (a raw C function
kept private behind a safe wrapper, good practice #5). Options: (a)
before the block only, (b) per line only, recommended, (c) both. Asked
"Isn't extern already public?": no, an extern is a name in its module,
private like any `fn` (checked in the oracle); "extern" says where the
body lives, `pub` who sees the name. The owner: "allow mixing, but block
overrides". So a line says its own `pub` / `pub.limb`, and a `pub` (or
`pub.limb`) before the block applies to every line, whatever the line
says. In the oracle; `examples/extern_block_pub.cpls`, reject test
`extern_block_line_private`; `extern_block_pub_inside` removed.

### 2026-10-02 — `extern.C`: it names its language, one line or a block (owner)

```
extern.C fn exit ['code': int32];
extern.C {
    fn malloc ['bytes': uint64] -> ptr.mut[uint8];
    fn free ['p': ptr.mut[uint8]];
    'lib_version': int32;
}
```

The owner: "Can we make it so that extern can become a block?" Design
claude asked whether the block comes beside the one-line form or replaces
it, recommending beside (one C function is common); the owner: "both".
Then whether the block names the language, (a) plain `extern { … }`,
since `extern` reaches only C, recommended, or (b) `extern.C { … }`, like
`layout.C`, which would change every one-line `extern fn` too. The owner:
"`extern.C`, and one line too". So `extern fn` and `extern 'x': T` are
now `extern.C fn` and `extern.C 'x': T`, and a bare `extern` or
`extern.c` is refused. Design claude's pick, changed the same day (above):
`pub` only before the block. In the oracle;
every example, reject test and design note rewritten; `low_level.cpls`
uses a block; reject tests `extern_no_language`, `extern_lowercase_c`,
`extern_block_pub_inside`, `extern_block_unclosed`. Entries below keep the
spelling of their day.

### 2026-10-02 — `inline.S` is another name for `inline.ASM` (owner)

The owner: "Add `S` as an alias to `ASM`". `S` as in assembly's `.S`
files; capitalized, as every language's name is (`inline.s` is refused,
naming `inline.S`). Design claude's pick, the owner's to change: `S` is
not reserved as a keyword, since it is read only after `inline.`, so a
struct or variable may still be called `S` and Synthia's keyword list
does not change. In the oracle; reject tests `asm_as_s`,
`asm_s_lowercase`; `ffi.md`.

### 2026-10-02 — A language's name is capitalized: `layout.c` becomes `layout.C` (owner)

The owner, after design claude compared `layout.c` with LuaJIT's
`ffi.cdef`: "Wait, why isn't it `layout.C`? I swear that I decided that
langs should be capitalizes". DECISIONS had only the 2026-09-20 ruling on
the `inline.` head ("the inline code name should be easy to spot, just
like `MAIN`"), a day after `layout.c` (2026-09-19). The owner: "Maybe it
wasn't recorded, look at session transcript". It was there: on
2026-10-01, designing `for !c`, the owner said "inline.C has a big c",
and design claude answered "Language names are capitalized in CyborgPL"
and recorded neither. Design claude asked whether that is the rule, so
`layout.c` becomes `layout.C` (`layout.C struct`, `layout.C enum`), and
recommended yes: one spelling per language, and `!c` stays visibly not
C. The owner: "BIG". In the oracle, every example, reject test and
`ffi.md`; `layout.c` is refused naming `layout.C` (reject test
`layout_lowercase_c`). Entries below keep the spelling of their day.

### 2026-10-02 — While something is lent `lend.mut`, nothing else may borrow it (owner: a)

```
two[lend.mut 'u', lend 'u'];             # refused: 'u' lent to change and lent again
two[grow[lend.mut 'u'], lend 'u'];       # refused too: grow hands back a borrow of 'u'
one[lend.mut 'u'];                        # to read and change: once, with lend.mut
let 'before' := copy 'u';                 # to keep how it was: copy first
two[lend.mut 'u', lend 'before'];
```

Compiler claude's question: the oracle and CCA both accepted the second
line, refusing only when `two` actually changed `'a'` while `'b'` was in
use; `ownership-syntax.md` called `lend.mut` "temporary exclusive access",
the 2026-09-14 rule said a borrow "never aliases a mutation". Proposed:
(a) refuse the call, as Rust does; (b) allow it unless a change actually
happens. Compiler claude and design claude recommended a. The owner asked
whether `two[lend.mut 'u', lend 'u'];` makes sense (design claude: no,
the two names are one list), whether (a) leaves a way to do it on purpose
(pass it once with `lend.mut`, or `copy` first), and the performance
difference (a lets the compiler tell LLVM the two never overlap; the
copy costs one allocation, and only when asked for). The owner: "`a`".
In the oracle, checked when the call runs; reject tests
`lend_mut_and_lend`, `lend_mut_via_return`. Two plain `lend`s of one
variable stay fine, and `list.append[lend.mut 'u', list.length[lend 'u']]`
too: the inner borrow ends before the call.

### 2026-10-02 — Check claude no longer runs Alive2; RRx2 replaces it (owner)

On the resume, design claude asked whether check claude should still run
Alive2, recommending no: RRx2 covers what it did, whole programs rather
than one function at a time, without Alive2's crash on Homebrew's LLVM 23
(check claude's workaround, running `opt` itself, had proven 211
rewrites, 0 wrong). The owner asked "Isn't Alive2 broken?" (its own
optimiser is; the workaround worked), then: "We have RRx2 to replace
ALive2, lol". `verification-session.md`.

### 2026-10-02 — Cores after the resume: 3 each again; RRx2-TV takes 8 only during a run (owner)

Design claude proposed the pre-pause split, 3 each for design, compiler
and check claude and 1 for the desktop, with RRx2-TV taking 8 during a
run the owner starts while compiler and check claude hold off on builds
and tests (RRx2 alone uses up to 18 GB of the 24; its 8 solver runs at
once had pushed the Mac 12 GB into swap). The owner: "Yeah, 3 for 3".
troll license claude stays idle (owner: "troll license claude will be
idle"), so it needed no share. `CLAUDE.md`.

### 2026-10-02 — CyborgPL resumes (owner)

Paused 2026-09-30 until RRx2-TV exists. Design claude asked whether
CyborgPL resumes now, meaning compiler claude, check claude and synthia
claude work again, with RRx2 idle until the owner says it is time. The
owner: "yea". The core split: above.

### 2026-10-02 — RRx2 runs when the owner says it is time, not after every CyborgPL change (owner)

The owner asked RRx2-TV claude what RRx2 covers (all 185 of CCA's test
programs pass its up-front check; of 154 run, 100 proven, 0 wrong), then
whether it is safe to continue CyborgPL. Design claude: yes by the
owner's bar ("It should cover what CyborgPL currently supports"), and
asked whether RRx2-TV keeps working beside the CyborgPL sessions. The
owner: "I think RRx2 will continue only when it's time to continue, not
after every single CyborgPL update." Whether CyborgPL resumes now was
asked next (above).

### 2026-10-02 — A bin into a `deci` says its form too: `to.stored.deci64` or `to.useful.deci64` (owner)

```
let 'x': bin64 = 0.1;
to.stored.deci128['x']    # 0.1000000000000000055511151231257827  (was to.deci128['x'])
to.useful.deci128['x']    # 0.1
```

The question the entry below left open. Design claude laid out the two
answers a bin has in a `deci` and asked whether the bare form should be
refused, as for print and text; recommended yes, the same surprise as a
bare print (write `0.1`, get 34 digits). The owner: "Ah, opt 1". A bare
`to.deci64[…]` of a `bin` is refused, naming both forms; design claude
counted `brain16` as a bin here (a binary number; `to.useful.deci` of a
`brain16` was the exact value until now and is now its shortest form,
`0.1`); the owner: "I think it's correct". Integers, `deci`s and text
into a `deci` stay bare. In the oracle
(reject test `deci_from_bin_bare`); `examples/deci.cpls`,
`examples/deci_context.cpls`, `types.md`, `conversion.md`.

### 2026-10-02 — A number into text says its form too: `to.stored.str.utf8` or `to.useful.str.utf8` (owner)

```
"#" ++ to.stored.str.utf8['n']     # 0.1000000000000000055…  (was to.str.utf8['n'])
"#" ++ to.useful.str.utf8['n']     # 0.1
to.str.utf8['bytes'] otherwise …   # bytes, text and characters stay bare: one form
```

Design claude asked whether `to.str.utf8` should need `.stored`, to match
the prints (left open by the entry below). The owner: "yes". A bare
`to.str.utf8[…]` (any text target) of a number, `bool` or enum case is
refused, naming both forms; `to.stored.` is accepted only before a text
target. In the oracle (reject test `to_text_bare`); examples, reject tests,
`conversion.md` and `syntax-sketch.md` rewritten. Whether `to.deci64[…]`
of a bin needs a form was not asked.

### 2026-10-02 — Every print says its form: `print.stored` or `print.useful`; `print.weird.stored` too (owner)

```
std::print.stored.stdout['x', \n];           # 0.1000000000000000055…   (was std::print.stdout)
std::print.useful.stdout['x', \n];           # 0.1
std::print.weird.stored.stdout <- 'x' <- \n;  # (was std::print.weird.stdout)
```

The owner: "Should we make `std::print.stored` mandatory, instead of
bare?" Design claude recommended it: a bare `print` showing 55 digits
for `0.1` is the most surprising default in the language, and with both
forms named nobody remembers which is the default; it touches about 135
calls; `print.weird` would follow; `to.str.utf8` (stored) against
`to.useful.str.utf8` has the same asymmetry and was left to ask apart.
The owner: "Though, it has to be required & compatible with `.weird`".
So a bare `std::print.stdout` / `.stderr` and `std::print.weird.stdout` are
refused, naming both forms; `.weird` stays first (`print.stored.weird` is
refused, the strongest modifier first). In the oracle; every example,
reject test and design sample rewritten (67 files). CCA, the bench and the
website still say `std::print.stdout`. `to.` was not asked.
`strings.md`, `syntax-sketch.md`.

### 2026-10-02 — `brain16` and `*0.1` are in the oracle

The owner: "Implement it". `brain16` (and `bfloat16`, `bf16`) is a type
and a value family of its own: literals round to the nearest; `+ - x
div mod xx` are done at 32 bits and rounded, as LLVM emits it, past the
largest a check site; comparisons work within the family; mixing with a
bin or integer is refused naming `to.`; `to.brain16[…]` from an integer,
a bin, a deci or text, and out of it to `bin32`/`bin64` (exact), `bin16`
(a check site past 65504), integers (`not_whole`, `out_of_range`) and
`deci`; `print` shows the exact value, `print.useful` and `{}` the
shortest that rounds back. Checked against a separate Python
computation (0.1 + 0.2 is 0.30078125). `*0.1` is a deci of the width the
place wants; under `:=` the oracle picks `deci64` (listed as its
provisional pick: the width is Precog's); in a bin or integer place it is
refused, as is `*5`. `examples/brain16.cpls`, `examples/exact_literal.cpls`;
reject tests brain_mixed_bin, brain_into_bin, brain_width, brain_overflow,
exact_into_bin, exact_integer.

### 2026-10-02 — `*0.1`: "exactly this", a `deci` (owner; not built yet)

```
let 'price' := *19.99;      # a deci: exactly 19.99
let 'speed' := 0.1;         # a bin
```

Working out whether a `:=` may ever become a `brain16`, design claude
found that no rule says which type an untyped decimal literal gets: the
oracle stores `0.1` as a 64-bit bin, but by default, not decision
(`bin16`, `bin32`, `bin64` each give a different value; only a `deci`
holds `0.1` exactly). The owner (asked earlier: "I thought we didn't
have lazy inferrence"): "`let 'x' := *0.1;` `*` means 'I want exactly
this', so it becomes a `deci` instead of a `bin`". Design claude noted
that a run of operator characters is one symbol, so `'a' + *0.1` needs
its space, that C and Rust readers know `*` as dereferencing, and that
C# and C23 use a suffix (`0.1m`, `0.1dd`); the owner: "`*0.1` is already
correct in CyborgPL" — kept as `*`. Design claude's proposals with it,
not ruled: `*` only before a decimal literal (integers are exact
already); Precog picks the `deci` width; a typed `deci` needs no `*`.

**A plain decimal `:=` means `bin64`'s answer; Precog narrows it only
where the whole program gives the same results.** The owner: "Doesn't
that depend on the entire program?" Design claude: yes, but bins of
different widths give different answers, so the width needs a fixed
meaning to compare against, or the same line would print differently
depending on the rest of the program; proposed `bin64` as that meaning,
narrowed wherever Precog proves a narrower width gives identical
results (`let 'half' := 0.5;` used only to halve powers of two can be a
`bin16`; a `0.1` that is printed or summed stays `bin64`), the integers'
"narrow when proven" rule applied to bins. The owner: "`let 'half' :=
0.5;` smallest that fits? If there's nothing else, of course". Among the
widths proven to give the same results, the pick is the 2026-09-14
"best, not smallest" rule's (the natural width for a lone scalar, the
narrow one for lists and structs). So `brain16`, which changes values,
is never chosen: only written. Not built yet. `types.md`, `precog.md`.



### 2026-10-02 — A brain float: `brain16`, also `bfloat16` and `bf16` (owner)

The owner asked whether LLVM supports bfloat ("Brain Float"): it does,
as the IR type `bfloat` (LLVM 11 onward); design claude compiled a
`bfloat` add for this Mac's M5 (`FEAT_BF16`), which LLVM widens to 32-bit,
adds, and narrows with the hardware `bfcvt`, since processors give
bfloat conversions and dot products, not general arithmetic. It has
`bin32`'s 8 exponent bits and 7 of precision (against `bin16`'s 5 and
10), made by Google Brain for neural networks. The owner: "Let's add it".
On the name, design claude noted `bin` means IEEE 754 binary and bfloat
is not IEEE, and offered `bfloat16`, `bin16.brain` or `brain16`,
recommending `bfloat16`. The owner: "`brain`, `bfloat`, and `bf`
(rs-style)", then, asked which is the main one, "`brain` is main, rhymes
too". So **`brain16`** is the type, with **`bfloat16`** and **`bf16`** as
aliases (bfloat comes in 16 bits only).

**Its arithmetic rounds after every step** (owner: "`a`"), as `bin16` and
`bin32` do (2026-09-24): each operation gives the nearest `brain16`,
which is what LLVM emits (widen, operate, `bfcvt`). Offered against it:
exact in between and rounded once when stored; no arithmetic, storage
only (compute in `bin32`, as mixed-precision AI code does).

**It is a family of its own** (owner: "`a`"), like `deci`: it never mixes
with a `bin` or an integer without `to.`, over joining the bins (mixed
freely, quietly widened; `brain16` + `bin16` would have no common type).
Conversions, design claude's sketch under the existing `to.` rules:
`to.bin32['w']` exact, never fails; `to.brain16['x']` rounds;
`to.bin16['w']` can overflow, a check site.

**A literal rounds to the nearest `brain16`**, as for bins (owner: "`a`,
but idk about Precog"): `0.1` is stored as `0.10009765625`; `print`
shows that, `print.useful` shows `0.1`. Over refusing any literal it
cannot hold exactly, which would refuse nearly all. Whether a `:=` may
ever become a `brain16` is open. Open too: the GPU. Not built yet.
`types.md`.

### 2026-10-02 — Type names: CyborgPL's own and Rust's, nothing else (owner)

The owner asked for long aliases too ("`binary_float16`,
`binary_float_16`, `binaryfloat16`, etc, etc"); design claude proposed a
rule (a long word per family, `_` or nothing between the parts) and asked
which families. The owner: "Actually, fuck it, only include the Rust ones
and ours". So the types are spelled CyborgPL's way or with Rust's 13
number names (`i8`–`i128`, `u8`–`u128`, `f16` `f32` `f64`); the C aliases
recorded earlier the same day are withdrawn, and no long names are added.
`types.md`.

Built the same day (owner: "implement now"): in the oracle a Rust name
is read as CyborgPL's wherever a type is written (declarations,
parameters, `list[i64]`, range types `i16[0, 100]`, `to.u16[…]`); `i32`
and the rest cannot name a struct, enum or interface; `usize` / `isize`,
`String` and `f128` are refused with what to write instead (design
claude's wording). `examples/rust_types.cpls`; reject tests rust_usize,
rust_string, struct_named_rust_alias.

### 2026-10-02 — C's exact type names are aliases too (owner; not built yet) — withdrawn the same day: only CyborgPL's names and Rust's

| C alias | CyborgPL type |
|---|---|
| `int8_t` `int16_t` `int32_t` `int64_t` | `int8` … `int64` |
| `uint8_t` `uint16_t` `uint32_t` `uint64_t` | `uint8` … `uint64` |
| `_Decimal32` `_Decimal64` `_Decimal128` | `deci32` `deci64` `deci128` (C23, IEEE 754 decimal) |
| `float` `double` | `bin32` `bin64` |

After the Rust aliases, the owner: "Now, what about C's?" Design claude
split C's names by how exact they are. Left out: `int`, `short`, `long`,
`long long` and the `unsigned` forms (sizes are the machine's: `long` is
64 bits on Mac and Linux, 32 on Windows; most are two words; "no bare
`int`" since 2026-09-14), `char` (its sign is the machine's), `long double`
(64, 80 or 128 bits), `size_t`, `bool` (the same word) and `__int128` (an
extension; standard C has no 128-bit integer). `float` / `double` are
the nearly exact pair: C does not require IEEE, but every target uses
it. Design claude leaned towards no C aliases, the `_t` names adding
nothing over CyborgPL's own; the owner, shown the everyday C types ("That's
awkward... For a lower-level lang") and then these: "record, don't
implement yet". `types.md`.

### 2026-10-02 — Rust's number type names are aliases (owner; not built yet)

| Rust alias | CyborgPL type |
|---|---|
| `i8` `i16` `i32` `i64` `i128` | `int8` … `int128` |
| `u8` `u16` `u32` `u64` `u128` | `uint8` … `uint128` |
| `f16` `f32` `f64` | `bin16` `bin32` `bin64` |

The owner: "I kinda want to add an alias for our types in Rust's syntax".
Design claude proposed the exact one-to-one matches and left out
`Vec<T>` / `HashMap` / `Option` (`< >` means a generic blank now),
`String` / `&str` (Rust's owned/borrowed split has no counterpart; one
`str.utf8`, lent with `lend`) and `usize` / `isize` (their size is the
machine's), noting the 2026-09-19 no-alias rule and its two exceptions
since (`&`, the long `print.useful` name). The owner dropped `bool`
(already the same word) and `char` ("is gonna confuse people": a bare
`char` is deliberately not a type, a unit must be said). Rust has no
decimal type (`rust_decimal` is its own 128-bit format, not IEEE 754), so
`deci` has none. The owner: "yea, don't implement yet". `types.md`.

### 2026-10-02 — The generics rules are in the oracle

Built in one go after the owner settled them: announcements after the
name for `fn`, `op`, `struct`, `enum` and `interface` (`<T>: Number`,
`<A>: any <B>: any`, `<T>: Number and Hashable`, `| @x @y`); a mention not
announced, a bound on a mention, a blank with no bound, `any` joined with
`and`, groups without the `|`, and an unannounced `@x` are refused; an
interface may list types one per line (matched by family in the oracle,
`int32` admitting any integer, as widths are the compiler's), alone or
beside operations; a struct's or enum's filled types are checked against
their bounds. Every example and reject test using generics or lifetime
groups was rewritten by hand; new `examples/interface_types.cpls` and nine
reject tests (`generic_unannounced`, `generic_bound_on_mention`,
`generic_no_bound` replacing `iface_two_bounds`, `generic_groups_no_bar`,
`generic_any_and`, `lifetime_unannounced`, `iface_type_not_listed`,
`iface_types_one_line`, `struct_bound_unkept`, `iface_bound_on_own_t`).
Also fixed on the way: four oracle messages still said `extern let`
(gone since 2026-09-25). `generics.md`, `interfaces.md`,
`ownership-syntax.md`, `maps.md`, `functions-as-values.md`.

### 2026-10-02 — An interface may list types, as well as operations (owner)

```
interface Number { int8 int16 int32 int64 bin32 bin64 }     # exactly these types
interface Ordered {                                          # anything with these operations
    op < ['a': <T>, 'b': <T>] -> bool;
    op == ['a': <T>, 'b': <T>] -> bool;
}
```

Working out how a generic says what it accepts, the owner asked whether
"anything" should be an interface naming every type; design claude
explained that interfaces list only operations today (the oracle refuses
`interface Number { int32 bin64 }`), so a type list is new, and that a
list of every type can never be finished, since each new struct is a new
type. The owner: "Bruh, that's why I was confused". Offered: a list of
types (exact, closed: a later `Money` can never join) against a list of
operations (open to future types); design claude recommended allowing
both, the writer picking per interface, as Go does. The owner: "yra,
both". Open: the spelling (the sketch above is design claude's), whether
one interface may mix the two, and how "any type" is said. Not built
yet; it goes into the oracle with the rest of the generics changes.
`interfaces.md`.

### 2026-10-02 — Generics and lifetime groups are declared up front, everywhere (owner: c)

Asked what Rust's up-front declarations are for, design claude gave the
reasons (telling a new name from an existing type, fixing the order for
filling blanks, scope, one place to read) and the cost of introducing by
first use: a typo (`<Y>` for `<T>`, `@y` for `@x`) silently makes a second
blank or group. The owner: "Hm, I mean. We could add them". Offered,
noting it reverses the 2026-09-20 choices of "both use `<>`" and "no
bracket, first appearance": (a) required in `#strict` files only,
recommended; (b) optional everywhere; (c) required everywhere, Rust's
way. The owner: "`c`". Open: the spelling of the list (design claude's
sketch, `fn max <T: Ordered, @x> […]`, `struct Pair <A, B> { … }`, was
not ruled), whether later mentions keep their `< >`, and where bounds go.
Nothing built yet. `generics.md`.

**And a generic says what it accepts; it never allows every type by
default.** Asked how a generic knows what it accepts, design claude
explained that a plain `<T>` takes any type the body works with, checked
at each call, and that a bound (`<T: Ordered>`) says it in the
signature. The owner: "But it's confusing, nobody defined `<T>` yet",
then "I meant that a generic needs defining first. Don't default to
allow all". Open: how a generic that truly takes any type (a `Pair`, the
first item of any list) says so.

**The definition goes right after the name, with what it accepts**
(owner: "`a`", over a `with <T: Number>` at the end, before the body):
`fn max <T: Number> ['a': <T>, 'b': <T>] -> <T> { … }`, `struct Pair <A,
B> { … }`, so `T` is introduced before its first use; later mentions keep
their `< >`, as shown. Whether lifetime groups join the same list
(`<T: Number, @x>`) was not asked.

**"Any type" is said with the built-in word `any`** (owner: "`a`", over
an empty interface each Limb writes itself): `struct Pair <A: any> <B:
any> { left: <A>  right: <B> }` (one bracket each, ruled just after).
Written, never assumed: a generic without a
bound is refused.

**Both ways of combining** (owner: "Both works"): one interface may mix
types and operations (`interface HashableNumber { int32 int64 bin64  fn
hash [lent 'v': <T>] -> uint64; }`, a type must be listed and have the
operations), and a generic may take several bounds joined with `and`
(`<T: Number and Hashable>`, every one must hold). Design claude had
recommended only the second.

**A type list goes down the page, one type per line**, as enum cases do
(owner: "Or just let it fucking go down"), after design claude's
one-line sketch `{ int8 int16 … }` read as one phrase and the owner asked
why there were no commas (fields and cases have none: line breaks
separate items in a `{ }` body, from 2026-09-18; commas there were never
asked). Offered against it: commas, or `or`.

```
interface Number {
    int8
    int16
    int32
    int64
}
```

**An interface announces its `<T>` too** (owner: "ye4", to design claude's
recommendation): `interface Ordered <T> { op < ['a': <T>, 'b': <T>] ->
bool; … }`, so the rule is one sentence — every generic is announced
right after the name, for functions, structs, enums and interfaces. A
type-list interface uses no `<T>` and announces nothing.

**One bracket per generic** (owner: "`b`"): `struct Pair <A: any> <B:
any> { left: <A>  right: <B> }`, so an announcement looks like its
mentions, one generic in each `< >`. Design claude had written `<A: any,
B: any>` in earlier sketches without asking; the owner chose this over
it. The `any` entry above is corrected to match.

**The bound goes after the bracket: `<A>: any`** (owner: "Wait a minute,
shouldn't it be `<A>: any`? I only choose `<>` for generics, because it
kinda looked like a placeholder", then "yea"). `<A>` is the placeholder,
the same in an announcement and a mention, and what it accepts follows a
`:`, as a type follows `'x':`:

```
fn max <T>: Number ['a': <T>, 'b': <T>] -> <T> { … }
struct Pair <A>: any <B>: any { left: <A>  right: <B> }
interface Ordered <T> { … }
```

This replaces `<T: Number>` (2026-09-20) in announcements; the earlier
examples in these entries keep the spelling they were written with.

**Lifetime groups are announced too, after the generics, and a `|`
separates the two kinds** (owner: "`fn pick <T>: any @x […]` a bit
confusing for me, so maybe add `|` to separate generics and lifetimes",
then, offered a bar between every item: "Not between every announcement,
because generics are still the same as generics, so as lifetimes"). Only
between the kinds; with one kind there is no bar. `< >` stays for
generics alone.

```
fn pick <T>: any | @x [lent@x 'xs': list[<T>]] -> lent@x <T> { … }
fn swap <A>: any <B>: any | @x @y [ … ]
fn longer @x [lent@x 'a': str.utf8, lent@x 'b': str.utf8] -> lent@x str.utf8 { … }
```

### 2026-10-02 — `&` works wherever `lent` does: `-> &T`, `&@x` (owner)

Asked whether `&` works as `lent`, design claude ran it in the oracle:
parameters, calls, function types and `return &'a'` worked; the return
type did not (`-> &str.utf8` was refused), since `&` (2026-09-21) predated
returned borrows (2026-09-29) and nobody had asked; nor did a lifetime
group on `&`. Design claude recommended covering every place, as a
spelling that works everywhere but one spot is a trap. The owner: "Yes."
So `-> &T`, `-> &mut T`, `-> &@x T`, `&@x 'a': T`, `&mut@x 'a': T`. In the
oracle (`&@` lexes as one operator, so the parser takes it as `&` then
the group); `examples/ampersand_everywhere.cpls`, reject test
ampersand_group_unnamed. `ownership-syntax.md`.

### 2026-10-02 — `let` is back on every declaration; `change` stays (owner)

```
let 'x' := 1000;
let mut 'sum': int64 = 0;
let !$ mut 'total' := 0;
let 'x' $:= 2000;            # shadowing
change 'sum' = 'sum' + 1;    # unchanged
```

The owner, shown the same program in today's syntax and as of
2026-09-24 (both run, one on an oracle rebuilt from 0561093): "Kinda want
`let` back (not the entire era), honestly. It just reads more
naturally." Design claude noted the `decl` idea of 2026-10-01 was the
same wish, dropped for the redesign around it rather than the word, and
recommended keeping `change`, so every statement that touches a variable
starts with what it does; the old era's bare `'x' = …;` change stays gone.
Nothing else from that era returns (`pos`, `/`, `>==`, `printu`). The
owner: "Show me first", then, shown the program with `let` and `change`,
"yeah". `mut` / `immut` / `!mut` sit between `let` and the name, `!$`
before them. Not asked, kept as today: `const` and `extern 'x': T` take
no `let`; loop counters, captures and `match` / `otherwise` names are not
declarations.

In the oracle: a declaration without `let` is refused naming it (reject
test let_missing, replacing let_removed; let_chain now refuses
`let.mut`). Every example and reject test, and the code samples and
quoted declarations in `design/`, gained `let` (1,010 declarations in
programs, by script). CCA, check claude's generator, the bench's
CyborgPL files and the website still use the 2026-09-25 form; CyborgPL is
paused, so they follow when it resumes. `syntax-sketch.md`.

### 2026-10-02 — `&` and `&mut` no longer punish (owner)

The owner: "`&` and `&mut` shouldn't punish anymore." The alternative
spellings of `lend` / `lend.mut` (2026-09-21) stay, with the same meaning,
but the build no longer prints `What are you? Rust programmer or Claude?`
once per `&`. Done in the oracle (the counter and the print are gone;
`examples/alternatives.out` lost its seven lines), `ownership-syntax.md`,
`diagnostics.md` and the oracle README. CCA still prints it
(`cca/compiler/src/main.rs`) and its run of `examples/alternatives` will
disagree with the new `.out` until compiler claude drops the print when
CyborgPL resumes.

### 2026-10-02 — Lilith gets a session and a public repo, design only (owner)

The owner, bored waiting on RRx2's tests: "I shall summon lilith claude".
Asked where it lives, what it does and in what: "Public under CyborgPL's
group, the CyborgPL signature warning verbatim, Apache 2.0 with LLVM
exception, local folder is anything you like, design for now". So the
public repo `Cyborg-Assembly-Manufacturing-Site/Lilith`, cloned at
`/Users/ts/Lilith` (design claude's pick of folder), README with
CyborgPL's "Not family friendly" warning word for word, Apache-2.0 WITH
LLVM-exception. Design claude kept the starting notes
(`secrets/lilith.md`) in this private repo and told lilith claude not to
copy them to the public one without the owner's yes; Lilith's own record
(where its decisions are written) is the session's to propose. Brief:
`design/lilith-session.md`. **The session is "lth design claude"**
(owner: "Changing the claude's name first, it's `lth design claude`"),
its commits ending `Co-Authored-By: lth design claude
<noreply@anthropic.com>`.

### 2026-10-02 — Quick Fix works from a spelling table, `spellings.cplr`, and each Limb's `cpl_ver.cplr` (owner)

Asked how Synthia's Quick Fix knows what to rewrite, design claude
offered (a) Synthia runs CCA and applies its errors' fixes, recommended,
or (b) its own Go list of rewrites. The owner: "Or... CyborgPL stores the
stable spelling, and maps the stable spelling to the current one?"
Design claude read it as a table of stable IDs and each version's
spelling, read by every tool (over files storing IDs, which would stop
being plain text), and noted that a table does renames only; the owner:
"Bruh, if `let` was dropped, just make it support that too", so a row may
spell nothing or add a word in a position. Asked for better ideas, design
claude added a version stamp per Limb, meaning changes listed rather than
rewritten, dependencies upgraded in a `.synthia/` copy, and a diff first.
The owner: "we could create a separate file if needed", "whatever is
good, bro", "we could create a new file type/extension if needed"
(design claude kept `.cplr`, CyborgPL syntax for tooling), then named the
stamp "`cpl_ver`", "with the dot": `cpl_ver.cplr`. `spellings.cplr` is
design claude's name. Asked whether CCA's package manager gets Quick Fix
too: design claude noted the 2026-09-29 rule (Synthia's edge is
convenience, never capability), that a CCA user could follow the errors
by hand except for meaning changes, which compile silently, and proposed
CCA noting those. The owner: "I think CCA's one should be a command, and
it tells you where & what to change. Synthia does it automatically after
a button or a CLI command". `tooling.md`.

### 2026-10-02 — What CyborgPL's IDE should have (owner)

The owner, after asking how moddable VS Code is: "What I would want the
IDE of CyborgPL to have (whether it's custom, or VS Code extension); 1.
Quick Fix (biggest one) 2. limb registry search 3. Solarized themes 4.
highlighting 5. Code picker (like Roblox Studio) 6. support C (though,
likely no Quick Fix for C) 7. support LLVM IR (no Quick Fix) 8. and other
good stuff". Quick Fix, design claude's first reading the editor's
lightbulb for a compiler error, is the owner's: "a way to fix your code
instantly (not literally) from update breaking your code" — rewriting
code to a new CyborgPL's spellings. Asked whether only an IDE button
or also a command: "a command", then "I think it should be Synthia. Just
like how Synthia can already change a limb's prefix, by Synthia itself
editing the source code" — a Synthia command; the name is open. Design claude did not know what the code picker was; the
owner had it screenshotted from Roblox Studio: the autocomplete popup with
kinds, a deprecated entry crossed out, and the signature, description and
documentation link of the chosen one. Then "record first, WITH the
screenshot". Open: custom or VS Code; doc comments, which the picker's
description needs. `tooling.md`, `design/images/roblox-studio-code-picker.png`.

### 2026-10-02 — Good practice is written down; names stay free (owner)

Asked what should be standard practice, design claude proposed twelve:
types at the doors and `:=` inside; outside values narrowed once with a
range type; no `mut` by habit; guard `if` then `check`, `nocheck` mostly
for C; `nocheck` wrapped once; `NO_FUCKING_CHECKS` only after measuring;
`#strict` for public Limbs; `lend` by default, `copy` rarely; failures
handled where understood, wary of `otherwise 0` on input; `and.then` for
guards; a plain `for`, with `seq`/`para`/`simd`/`!c` only for a reason;
`print.useful` for people, `print` for debugging, `print.format` for
shape. It asked whether to pick a naming style. The owner: "we don't need
a guide on how to name vars. But, all that you said there seems like a
good practice", then "write". Advice, not rules. `design/style.md`.

### 2026-10-01 — RRx2's exact-math language is called RR (owner)

The owner, asking whether it would be in C: "`RR` (the name of
RRx2-lang)". Design claude asked whether that made `RR` the language's
name, with RRx2-lang only the folder and the session; the owner: "yea, RR
is the name". Programs stay `.RR` (either case). The session is still
RRx2-lang claude, working in `lang/`. `rrx2-lang-session.md`.

**And the validator is RRx2** (owner: "`RRx2` is the TV"). Design claude
put back the reading: the validator RRx2 (no "-TV"), the language RR, the
README's line "Verifiers for CyborgPL: RRx2™ and RR™", the repo keeping
its name `RRx2`, and the sessions keeping theirs (RRx2-TV claude,
RRx2-lang claude); the owner: "yea". The marks are now RRx2™ and RR™.
The RRx2 README and `lang/README.md` are changed; `tv/README.md`, RRx2-TV
claude's, is left for it.

### 2026-10-01 — `inline.MSL` names a slot as Metal does: `buffer(0)` (owner)

The right side of a binding is the slot's name in Metal, copied exactly,
so `with [lend.mut 'xs' -> buffer(0)]` matches the kernel's
`[[buffer(0)]]` letter for letter, as `inline.ASM` writes `x0` the way
ARM does. Design claude offered (a) `buffer(0)`, recommended; (b)
`buffer 0`; (c) `buffer[0]`, noting CyborgPL's `( )` otherwise only
group; the owner: "what", then, put plainly, "okay". The `inline.MSL`
block is now spelled in full:

```
nocheck inline.MSL threads 'n' { … } with [lend.mut 'xs' -> buffer(0)];
```

`gpu.md`.

### 2026-10-01 — `inline.MSL` binds a lent list with `->`, as `inline.ASM` does (owner: a)

```
nocheck inline.MSL threads 'n' { … } with [lend.mut 'xs' -> buffer(0)];
```

Design claude's sketch had `lend.mut 'xs' <-> buffer(0)`, written
without checking `ffi.md`: in `inline.ASM` a lend goes in with `->`,
`lend.mut` says the code may write through it, and `<->` after a lend is
refused (owner, 2026-09-30: a) — `->`/`<-` move a plain variable's value,
while a lend only passes access to data used in place. Offered (a) `->`,
as `inline.ASM`, recommended, or (b) `<->`, the sketch. The owner asked
why `<->` had been given (design claude had pictured the data's trip to
the GPU and back, which is what `<->` means for a plain variable), then:
"`a`". Still open then: `buffer(0)` and its parentheses (settled next).
`gpu.md`.

### 2026-10-01 — `inline.MSL`'s thread count goes after the name: `inline.MSL threads 'n' { … }` (owner: b)

```
nocheck inline.MSL threads list.length[lend 'xs'] {
    kernel void twice(device int *xs [[buffer(0)]], uint id [[thread_position_in_grid]]) {
        xs[id] = xs[id] * 2;
    }
} with [lend.mut 'xs' -> buffer(0)];      # `->`: decided after, see above
```

The thread count, how many copies of the kernel the GPU runs (in plain
Metal, the host's `dispatchThreads` call), is any number: a literal, a
variable, or worked out on the spot. Design claude offered (a) in the
`with` list, beside the bindings, as `inline.ASM` keeps `clobber x16`
there; (b) after `inline.MSL`, bare, before the code; and recommended b:
how the block runs is read first, as `for para` puts its modifier before
the body, `with` keeps one job (what the block touches), and no `[ ]` is
needed where nothing is built (2026-09-19). The owner asked whether
`inline.MSL` is Metal's syntax (the code inside the braces is; the header,
`threads` and `with` are CyborgPL, standing in for the host program), why
the count is a variable (it need not be), whether `clobber` should move
too (no: it is something the block touches, so `inline.ASM` is
unchanged), and whether `threads` inside the braces makes sense (no: the
braces go to Apple's compiler untouched, and a shader cannot set its own
count). The owner: "Okay, `b`". Still open then: the binding list's
spelling (design claude's sketch had `<->`; settled next as `->`). `gpu.md`.

### 2026-10-01 — The `decl` redesign is dropped; the 2026-09-25 syntax stays (owner)

The owner, after the function-type question ("wtf are fn types?"): "Nvm,
let's just go back to the no `let` era". The whole `decl` redesign below
is withdrawn: `decl`, `mut`/`immut`/`!mut`/`!$` after the colon, `|`
between parameters, `const 'max': pub`, and the quoted-name rule. What
stays is the 2026-09-25 form: `'x' := 1;`, `mut 'x': int64 = 0;`,
`!$ mut 'total' := 0;`, `'x' $:= 2;`, `change`, `pub const`, commas in
every bracket list. Nothing had been built, so only `DECISIONS.md` and
`syntax-sketch.md` change. The redesign's entry stays below as history.

### 2026-10-01 — `decl` starts a declaration; `mut` goes after the colon; `|` separates lists that hold types (owner) — reverted the same day: the 2026-09-25 syntax stays

```
decl 'x' := 1000;                     # was  'x' := 1000;
decl 'x': int32 = 1000;               # was  'x': int32 = 1000;
decl 'total': mut, int64 = 0;         # was  mut 'total': int64 = 0;
decl 'n': mut := 0;                   # was  mut 'n' := 0;
change 'total' = 'total' + 1;         # unchanged

fn grow ['xs': mut, list[int32] | 'n': int64] { … }      # was  ['xs': list[int32], 'n': int64]
fn total [lent 'xs': list[int64] | 'keep': fn[int64] -> bool] -> int64 { … }
extern fn write ['fd': int32 | 'buf': ptr[uint8] | 'len': uint64];
fn[lent Locker | lent Locker] -> bool                    # a function type

grow[lend.mut 'xs', 5];               # calls keep commas
decl 'xs' := [1, 2, 3];               # lists keep commas
Locker[name = "A", size = 3]          # building a struct keeps commas
```

The owner: "Honestly, I might change no `let` to `decl`". Design claude
set out the case: every statement that touches a variable would start
with a word (`decl` creates, `change` changes), declarations become easy
to search for; against, it is the third swing (`let` added 2026-09-19,
removed 2026-09-25) and the churn, counted at 1,783 declaration lines in
572 program files plus `design/` and the website. Then the owner:
"`decl 'total': mut, int64 = 0;` might be the new one, honestly" — `mut`
moves out of the front into the description after the colon. Without a
type, design claude offered `decl 'x': mut := 1;` (one rule: the name,
then everything about it after the colon) against `decl mut 'x' := 1;`
and recommended the first. Owner: "yea, `decl 'x': mut := 1;`".

The comma after `mut` clashes with the commas between parameters
(`['xs': mut, list[int32], 'n': int64]` reads as three items). Design
claude recommended a space instead (`mut int64`). The owner: "Or swap
`fn`'s `,` with `|`", then "Consistent", and, asked which lists, "The ones
that clash with `,`": the lists that hold **types** — function
definitions, `extern fn` definitions and function types. Lists of
**values** — calls, list literals, building a struct — keep commas. So a
definition (`|`) and its call (`,`) now look different, which the
2026-09-19 "at least the insides look alike" had avoided. Struct fields
are one per line, with no separator, and do not change.

Borrow modes on parameters stay in front of the name, `lent 'xs':
list[int64]`; design claude asked whether they should move after the
colon like `mut`. Owner: "stay in front, record".

Design claude notes, as its own: `|` stays free as a custom operator on
values (`operators.md`), since lists that hold types contain no
expressions.

**`immut` and `!mut` go after the colon too**, where `mut` went:
`decl 'limit': immut := 10;`, `decl 'limit': !mut := 10;`. Design claude
asked (after the colon, recommended, or in front); the owner: "Lol, bro,
`mut`'s already behind".

**`!$` goes after the colon as well**, first in the list:
`decl 'total': !$, mut, int64 = 0;`, `decl 'c': !$ := 1;`. Design claude
offered that (recommended) or `decl !$ 'total': …` in front of the name;
the owner: "Behind".

**Shadowing keeps its `$` on the `:=` / `=`**, with `decl` in front:
`decl 'x' $:= 2000;`, `decl 'x': int32 $= 2000;`, `decl 'x': mut, int32
$= 2000;`. Design claude first recommended `$` after the colon
(`decl 'x': $ := 2000;`), beside `!$`; the owner checked that `$=` means
"this line shadows", not "can be shadowed", and design claude switched:
after the colon it would read as a property of the variable, while
shadowing is something the line does. Design claude confirmed from the
2026-09-14 entry and the oracle (an unmarked redeclaration and a `$:=`
with nothing to shadow are both refused). The owner: "The current one
makes sense, right?", then "yea, record a".

**`const` stays its own starting word, without `decl`, and `pub` goes
after its colon**: `const 'max': pub := 8;`, `const 'max': pub, int32 =
8;`, `const 'inner': pub.limb := 8;`. Design claude recommended keeping
`pub const` as it is, then explained why `pub decl` cannot happen (`decl`
is only inside bodies, `pub` only at the top level). The owner: "const
'max': pub?". Design claude noted `pub fn`, `pub struct` and `pub.limb
module` have no colon to move behind and recommended one place for
`pub`; the owner: "I mean, `const` is kinda like a var, though." Design
claude then put it as a rule: **quoted names** (variables, parameters,
consts) — the name, then everything about it after the colon, `pub`
included; **bare names** (`fn`, `struct`, `module`) — `pub` in front,
having no colon. The owner: "Ah, yes. I think your rule is good."

**No `decl var` / `decl const`, and no `var` on parameters.** The owner
asked: "Should it be `decl var 'x'` & `decl const 'x'`? And the fn params
could be like `var 'x'`, if it's actually a variable. idk". Design claude
recommended against: `decl` (only inside bodies) and the quoted name
already say "variable", and `const` alone already says "constant"; a
named kind would be `var 'x'` instead of `decl`, not beside it; an owning
parameter is a variable, but "owning is bare" (2026-09-19) already marks
it, and `var` would sit on nearly every parameter. The owner: "keep".

**C variables and owning parameters follow the same rule**:
`extern 'lib_verbose': mut, int32;` (was `extern mut 'lib_verbose':
int32;`), with `extern` as the starting word as `const` is, and `fn f
['n': immut, int64]` (was `[immut 'n': int64]`). `extern fn` has a bare
name and changes only by `|`. Design claude proposed both as following
from the rule; the owner asked "Is it extern or extern fn?" (both exist;
only the variable form was in question), then "yea, record both."

**Calls keep commas, confirmed.** The owner later asked "isn't it
`grow[lend.mut 'list' | 5];`?". Design claude owned that "calls keep
commas" had been its own narrowing of "The ones that clash with `,`",
never said by the owner, and asked again. Asked what made more sense,
design claude recommended commas: they never clashed in a call (which
holds values, as a list does); `|` in calls would take it from custom
operators (`f['a' | 'b']` would read two ways); and every call would
change. The owner: "Hm... I think you might be right, because
`std::print.useful.stdout["total: " | 'total' | \n];` genuinely looks
like it's seperated, even though it isn't".

Every declaration question is now ruled. Nothing is built yet: the oracle, `examples/`, the reject tests and
`design/` samples still use the 2026-09-25 form, and CyborgPL is paused.
`syntax-sketch.md`.

### 2026-10-01 — RRx2-TV judges threaded code piece by piece, plus a check that LLVM adds no data race (owner: b)

RRx2-TV claude's question: CyborgPL's parallel `for` and `spawn` run
program code on several threads at once. Proposed (a) run the pieces one
after another in its model — simplest, but it would miss an LLVM bug that
only shows when pieces run together, such as LLVM adding a data race, so
a "proven" could be wrong; (b) the same, plus a check that the optimised
code adds no data race the original did not have; (c) leave threaded
programs "not judged" for now. RRx2-TV claude recommended b. The owner:
"`b`". RRx2-TV claude adds, as its own: until that is built, threaded
programs stay "not judged", and callbacks on the same thread (map specs,
`qsort`) come first.

### 2026-10-01 — The first public GitHub release needs only Metal on the GPU side (owner)

"Btw, first pub GH Release only has to support Metal in GPU". Vulkan stays
a v1 target (2026-09-21) and comes after Metal (same day), but the first
public release does not wait for it. `gpu.md`.

### 2026-10-01 — The std Limb makes low-level things easier; higher-level things are other Limbs (owner)

Asked what std should hold, design claude listed what is already `std::`
(the prints, `input`, `exit`, rounding, `div.trunc`/`mod.trunc`, `uenr`,
`str.*`, `time`, `duration`) and the usual missing ground (files,
arguments and environment, collections, sorting, random, maths,
networking, parsing and JSON). The owner: "I think the stdlimb should just
make lower-level things easier". Design claude read it as a thin layer
over what the machine and the operating system already offer — what a
program could do itself with `extern fn` and `inline.ASM` — such as
files and folders, arguments and environment, memory, time, printing and
input, processes; and JSON, HTTP and network protocols, big data
structures and higher maths left to other Limbs from the registry. The
owner: "yea, though it might want to have GPU stuff". So std also holds a
low-level GPU layer — explicit control beside the automatic GPU loops,
which stay syntax-free (`gpu.md`). What it holds: design claude offered
(a) looking only — which GPU, its memory and cores; (b) explicit control
— moving data to GPU memory and back, and forcing a loop onto the GPU
over Precog's choice, as `for para` forces cores; (c) raw GPU programs,
Metal Shading Language written and launched by hand, a `nocheck` world as
`inline.ASM` is; and recommended a and b. The owner: "`b` & `c`". Raw GPU
code is written **inline**, a block like `inline.ASM`, so the Metal
toolchain checks it at build time (owner: "inline", over a text string
given to std) — and, the owner: "shouldn't it be in the base lang?": yes,
as `limbs.md` already has `extern`, pointers and inline asm in the base
language with std only "the library everyone uses for them". So the
inline GPU block is base language, `nocheck`; std holds the easy parts
around it — moving data, launching a block with N threads, forcing a loop
onto the GPU. **The block is `inline.MSL`** (owner: "`MSL`", over
`inline.METAL`): the family names the language inside, as `ASM` does, and
Metal is also the API's name. Open: the other spellings, std's full
contents, and Vulkan's block when Vulkan comes.
**An `inline.MSL` block runs where it is written**, as `inline.ASM`
does — not a named kernel that std launches later (owner: "It doesn't
need std"; design claude had recommended the same). Running it many
times is a function that holds it; the Metal code is compiled once, at
build time, and each run launches it. Design claude's sketch:
`nocheck inline.MSL { kernel void … } with [lend.mut 'xs' <-> buffer(0)],
threads …;` — the binding list and the thread count's place still open
(the count: after the name, owner, the same day).

### 2026-10-01 — CyborgPL is unstable, forever (owner)

Design claude noted that CyborgPL can rename freely (`/` → `div`, every
use converted the same day) because nobody outside the project depends on
it yet, not because of Apache 2.0's "AS IS" — C++ is held still by its
users, not its licence — and that languages usually state a policy, as
Rust's "anything may change before 1.0, nothing that breaks code after".
The owner: "CyborgPL will be, and I quote myself 'unstable' forever". So
there is no stability promise, before or after any version: anything may
change, and old spellings are refused with errors naming the new ones,
as they are now.

### 2026-10-01 — `std::print.weird`: C++'s `cout <<` with `<-` arrows (owner)

The owner, after asking how C++ prints: "Let's design a C++ print".
Design claude noted the borrowed prints of 2026-09-19 (removed the next
day) and that `<<` is free (shifts are `shl`/`shr`), and offered (a)
the arrow style or (b) C++23's `std::println` (nearly `print.format`
already). The owner: "`std::print.weird` seems like a good name, maybe
use `->` arrows". Design claude: `->` would point away from the output,
against "the arrow points where the value goes" (return types,
`inline.ASM`); `<-` points in, like C++'s `<<`. The owner: "`<-`". Form:
design claude offered stored or readable; the owner: "what if we add a
way to use `.useful` with `.weird`?" — `.weird` stored as `print`,
`.weird.useful` readable as `print.useful`, `.weird` first by the
strongest-first rule (owner: "yea"). `design/strings.md`;
`examples/print_weird.cpls`; reject tests print_weird_brackets,
print_weird_order.

### 2026-10-01 — GPU loops: Metal is built before Vulkan (owner)

"Btw, it's gonna be Metal before Vulkan." Both stay the v1 targets
(2026-09-21); the order is Metal first — CCA's own Metal Shading Language
text — then Vulkan through LLVM's SPIR-V. Metal is the GPU on this
machine, so it can be built and tested here. `gpu.md`.

### 2026-10-01 — "Rust throughout" is about code we write; borrowing Clang inside CCA is fine (owner)

Asked what `inline.C` would bring (C's headers and macros, no separate
files, inlining across the boundary through one LLVM module, Clang's
built-ins; every block a `nocheck` site), design claude listed as a cost
that Clang is a large C++ library, against "Rust throughout, C++ only
for the LLVM wrapper". The owner: "Clang inside CCA is fine. Like, mf, we
ain't writing Clang in Rust". So the rule (`implementation-languages.md`)
governs the code this project writes; a borrowed library in another
language — LLVM already, Clang if `inline.C` is built — is a dependency,
not a breach. `inline.C` itself stays "feasible later" (`ffi.md`).

### 2026-10-01 — `for !c`, "no cheat": a loop whose every round really runs; strongest modifier first (owner)

The owner's "cheating" benchmark (N baked into the source) showed clang
folding `acc += i` over 1..300 000 000 to `mov w0, #128; ret`, and CCA's
sliced loop folding the same way (0.00 s); an empty `inline.ASM` passing
'acc' through a register forced the real run (0.50 s). The owner: "Is
there a way to always run the loop in CyborgPL?", then "let's design it".
Design claude offered (a) a value-level barrier, as Rust's `black_box`
(recommended); (b) a loop word; (c) a block. The owner: "`for !c` means
nocheat". What it forbids: (1) skipping work — folding to the answer,
dropping rounds — while SIMD and cores stay allowed (recommended); (2)
any change at all. The owner: "Skipping work, because `seq` could be used
with `!c`". Word order: design claude first suggested `seq`/`para`
before `!c`, then, when the owner said "They both modify hard, like
`.useful`", the `.useful` rule — the modifier that changes the action
most sits next to it — which puts `!c` first. The owner: "Ah yes, so the
strongest modifer comes firts?", "yeah, strongest first": a general rule,
`for !c seq`, `for !c para`. Open: whether `while` and `loop` take `!c`
too. `parallelism.md`; `examples/for_no_cheat.cpls`; reject test
for_no_cheat_order.

### 2026-10-01 — RRx2-lang's file extension is `.RR`, either case accepted (owner)

The owner asked about `.rx2`; design claude noted it is already ReCycle's
REX2 audio-loop format (Reason, Ableton) and offered `.rrx2` and `.rr2`,
recommending `.rrx2`. The owner asked about `.rr` (no common clash known;
`rr` is Mozilla's debugger, a program not a file type), then whether it
could be capitalized: yes, as R's `.R` and assembly's `.S`, but macOS and
Windows usually ignore case in file names and Linux does not (C's `.C`
for C++ against `.c` is the cautionary tale), so the question was
whether `.rr` is accepted too. The owner: "`.RR`, accept either case.
Mimics `.R`". RRx2-lang session brief, open question 2.

### 2026-10-01 — RRx2-TV may use CCA's own test programs as inputs (owner)

RRx2-TV claude asked: "May I use CCA's test programs as inputs? I'd run
CCA on them and read the IR it produces and the programs themselves,
never CCA's own code", and recommended yes: "cover what CyborgPL
currently supports" needs a measure of what CCA compiles, and its tests
are the widest set. The owner: "yes, use them". The rule against reading
`cca/` otherwise stands: only the `.cpls` test programs under
`cca/compiler/tests/` and the IR CCA writes from them. Design claude adds:
those programs' comments carry expected answers written by CCA's author,
so they are inputs, not truth.

### 2026-10-01 — "RRx2-TV exists" when it covers what CyborgPL supports today (owner)

The RRx2-TV brief's question 2, which decides when CyborgPL resumes.
RRx2-TV claude offered (A) soon, after a soundness review, with coverage
growing once CyborgPL resumes; (B) later, a verdict on nearly every
program CCA builds, decimals and threads included; (C) something else;
and recommended A. The owner: "It should cover what CyborgPL currently
supports". RRx2-TV claude's reading, not yet the owner's words: every
feature CCA compiles at the pause (f497869) is modelled, so no program
is refused for an unmodelled feature.

Status then (RRx2-TV claude's runs): of check claude's corpus, 73
programs, CCA builds 61; RRx2-TV takes 47 — 43 proven, 3 not judged (2
recurse deeper than it follows, 1 needs more than 4,096 outside calls), 1
undecided (solver time), 0 wrong — and refuses 14 for features not yet
modelled: `bin` floats, SIMD vectors, threads and parallel `for`, function
pointers, a local's address reaching the runtime. RRx2 5245afc fixed a
soundness hole (a write into `getenv`'s string kept `strlen`'s old length,
so a planted bug came out "proven"); no verdict had depended on it.

### 2026-10-01 — LNIR is built with GPU loops, and never makes a program slower (owner)

The owner: "Could we have LNIR when we do GPU loops? It's a hobby lang,
and I think that's more fun. Only if it doesn't reduce perf, though".
Design claude: an IR costs nothing at run time (it exists only while
compiling, and compile time is free); only a bad transformation could
cost speed, so the rule is that LNIR changes a loop only when Precog's
estimate or a `tune.cplr` measurement says the change wins, and passes it
through untouched otherwise — the worst case is the program without
LNIR, and the bench can build both ways to check. Replaces "a fourth IR
only if offload gets serious". `gpu.md`, Open.

### 2026-10-01 — The loop-nest IR, if built, is named LNIR, *Loop-Nest IR* (owner)

Design claude proposed LNIR, NIR (Nest IR), Lattice, Loom and Grid, and
recommended Loom. The owner first chose NIR ("NIR, I guess"; Mesa, the
Linux GPU drivers, already has a shader IR called NIR), then floated
`GIR`, GPU IR; design claude noted the IR would serve CPU loops too
(tiling for the cache), so a GPU name undersells it, and that GIR is
GObject Introspection's format. The owner: "Switch to `LNIR`". `gpu.md`,
Open.

### 2026-10-01 — A fourth, loop-nest IR, if one is ever built, is our own, not MLIR (owner)

The owner asked how many IRs CyborgPL has (three: AST/typed AST, MIR,
LLVM's — 2026-09-22), whether that is enough (yes, the shape of Swift's
AST → SIL → LLVM; nothing in CCA has needed a fourth), and what the next
would be: the loop-nest (polyhedral) IR `gpu.md` names for GPU tiling,
fusion and thread mapping, between MIR + proofs and the Metal / SPIR-V
output. Proposed: MLIR (mature passes, likely faster programs sooner;
C++; its loop analysis sees only simple shapes; no Metal output) or our
own (starts behind; reads Precog's proofs directly, so it can go further
on loops MLIR cannot analyse; Rust). Design claude gave no number —
building both is the only honest measure — and noted the motto fits our
own. The owner: "our own". Still only if GPU work proves MIR is not
enough. `gpu.md`, Open.

### 2026-10-01 — `std::print.format` uses `{}` holes, with its own escapes read in one pass (owner)

Supersedes the two entries below of the same day (`%` codes, `\%`). The
owner asked how every type's `%` code should look; design claude laid
out a full table (C's letters, plus `%b`, `%t`, `%v`, `deci` under `%f`).
Piece by piece: `%b` for base 2 raised "binary or `bin`?"; the owner
weighed renaming `bin`/`deci` to `bfp`/`dfp` and kept them ("keep them";
IEEE 754 names them binary64/decimal64, so both names are accurate, and
they appear ~1,100 times in 123 files); asked for `%bin`, which design
claude showed is ambiguous after `%b`; asked whether CyborgPL's writers
are C people — design claude: probably not mostly, so C's letters are not
owed — and "it's not like it's called `std::c.printf`". Then: "Or just go
with {} holes?" The owner: "yea, {} holes". The six questions after:
1. a bare `{}` prints the **readable** form, as `print.useful` (owner:
   "If you can change it inside, then go with readable");
2. shapes are Rust's set — fill/align, `+`, `0`, width, `.places`, kinds
   `x X o b e` — plus **`?` for the stored form**, and **without `#`**
   (owner asked "doesn't `#` mean comment… `#strict`", then about another
   symbol — only `%` was free — then "drop it": the prefix is typed);
3. literal braces are **`\{` `\}`** (owner: "`\`");
4. `\%` is refused, unknown escapes stay errors (owner: "Fine");
5. **numbered holes** allowed (owner: "allow num"), **counted from 1**
   (owner: "Position 0 doesn't exist, it's before 1"), **never mixed** with
   plain holes (owner: "no mixing");
6. **lists, structs and maps** fill `{}` and `{:?}` (owner: "allow").
Testing it, design claude found that backslashes were read twice — the
language's text rules (`\\` → `\`), then the format's — so a literal
backslash took `\\\\`, and its own earlier answer (`\\%`) was wrong. The
owner: "Format print has its own rules": the format literal is read **as
typed, in one pass**, with only its own escapes; a non-literal (`nocheck`)
format is read as stored. `{:.2}` of a `deci` now prints `465.00`, the
question this began with. Oracle: `hole_format`; `design/strings.md`;
`examples/print_format.cpls`; ten `print_format_*` reject tests (`%u` and
`%%` tests removed).

### 2026-10-01 — In `print.format`, a literal percent is `\%`, never `%%` (owner) — superseded the same day: `{}` holes, `%` plain text

The owner: "Why not `\%` for literal `%`?" Design claude: C doubles it
only because C's backslash escapes are resolved when the string is read,
before `printf` ever sees it; `print.format` handles its escapes and its
codes itself, in one pass, so `\%` is doable and matches backslash meaning
"literal" (`\"`, `\\`). Recommended `\%` only, with `%%` refused. The
owner: "`\%` only", and asked how to write two: `\%\%`. The oracle
refuses `%%` with that hint (reject test print_format_double_percent);
`examples/print_format.cpls` uses `\%`; `design/strings.md`.

### 2026-10-01 — `std::print.format` stays C's codes with `%` (owner) — superseded the same day: `{}` holes

The owner, finding `print.format` refuses a `deci` (`%.2f` cannot print
`$465.00`): "let's redesign `std::print.format` entirely". Proposed (a)
codes in the text, as C; (b) holes in the text (`{'price':.2}`), which
would break the rule that quotes hold exactly what is stored; (c) the
values as a list, as in the other prints, with a shape word around a value
that needs one. Design claude recommended c; the owner first heard it as
folding formatting into plain `print` ("print is print, print useful is
print useful… print in CyborgPL is pretty literal"), so `print.format`
keeps its own job. The owner: "`a`, but we'll have to choose the less
used symbol to replace `%`". Design claude counted candidates in ~300,000
characters of local text (`^` 8, `$` 14, `@` 19, `&` 27, `%` 42, `~` 51,
`|` 188, `#` 793) and recommended `^`. The owner chose `@` ("it's really
just a matter of what's faster"), then, before it was committed: "`@` is
used" (lifetime groups, `lent@x`), then "maybe we'll just go `%`".
`%` is no CyborgPL operator (remainder is `mod`), so nothing clashes.
`print.format` is unchanged; a literal `%` stays `%%`. Still open:
whether `%f`/`%e`/`%g` take a `deci`.

### 2026-10-01 — RRx2-TV builds with CMake and Ninja (owner)

The owner asked design claude what CMake was and whether it would be "a
lot better" than RRx2-TV's hand-written Makefile; design claude said not
yet, worth it once the build got painful (two builds, finding LLVM). The
owner then asked RRx2-TV claude: "Should we use Cmake?", then "Switch now"
(RRx2 b689699). LLVM's flags come from LLVM's own CMake file; the
`test` preset builds with the sanitizers and the `final` one without, each
at most 8 jobs; `ctest` runs the known answers and the Alive2 comparison.

### 2026-10-01 — RRx2-TV is written from nothing; it reads LLVM's manual, tests and IR library, never the optimiser's meanings (owner)

RRx2-TV claude's first question. Proposed (a) start from Alive2's code;
(b) write from nothing and never read Alive2's code, keeping Alive2 as an
independent cross-check; (c) from nothing, reading Alive2's code where
LLVM's manual is unclear. RRx2-TV claude recommended b. The owner: "Could
you do `b`, but look at LLVM's source code?" RRx2-TV claude split LLVM's
source: it reads LLVM's Language Reference, LLVM's test files (as known
answers, not taken on trust), and the C++ library it uses to read IR; it
does not read the optimiser's code to learn what instructions mean,
since that is the code it judges and it would copy its mistakes. Reading
the optimiser only to see what shapes its output takes is allowed: a
mistake there can only cost "not judged", never a false "proven". Where
the manual is unclear: the stricter reading, or "not judged". The owner:
"`B plus the manual, tests and library`".

Found on the way (RRx2-TV claude, this session): **Alive2 at its default
(`--src-unroll=0`) passed a planted rewrite that is wrong only when a loop
runs more than 5 rounds**; it passed at unroll 2 and 4 too, and caught
it at 8. Check claude's 93c6e49 run used the default, so its 211
"correct" hold only for runs that never repeat a loop, in any function
with one. Also: only 1 of the 121 examples builds with CCA today (the
rest need `std`); check claude's corpus builds 61 of 73 (12 refused on
purpose); LLVM 23.1.2 parses all 124 of CCA's before/after files (CCA
emits LLVM 22.1.8); and the only facts Precog hands LLVM there are
`nsw`/`nuw` and `getelementptr inbounds nuw` (no `!range`, no
`llvm.assume`).

### 2026-10-01 — RRx2-TV's final build has no checkers; RRx2-lang keeps them always (owner)

Corrects the 2026-09-30 entry below, which design claude recorded as
"the checkers stay on everywhere, always" for both tools. The owner: "Ha,
I said 2x slower for the lang" — the speed point was RRx2-lang's, which
"doesn't give a flying fuck about runtime perf". For RRx2-TV, design
claude recommended keeping them on (most of its time is Bitwuzla, which
they do not slow), then deciding by measuring both builds; the owner,
noting its runs "might just be hella longer than our current oracle
testings": "Nah, no final check". So RRx2-TV builds with
AddressSanitizer and UndefinedBehaviorSanitizer for every test run and
without them for its final build; RRx2-lang keeps them in every build.
Warnings as errors stay for both.

### 2026-09-30 — "RRx2 VC" is now just "RRx2" (owner)

"Change RRx2 VC to RRx2". The repo is renamed
`Cyborg-Assembly-Manufacturing-Site/RRx2` (GitHub redirects the old
`RRx2-VC` address); its README title is "RRx2™", its line "Verifiers for
CyborgPL: RRx2-TV™ and RRx2-lang™"; the local clones are `/Users/ts/RRx2
lang` and `/Users/ts/RRx2 tv`. RRx2-TV and RRx2-lang keep their names.
Earlier entries say "RRx2 VC" as it was then.

### 2026-09-30 — RRx2's C and C++ keep Clang's checkers on in every build, the final one too (owner)

Design claude's safety net for RRx2 in C and C++: AddressSanitizer and
UndefinedBehaviorSanitizer on test builds, warnings as errors, Clang's
static analyzer, fuzzing with libFuzzer, and RRx2-lang's arithmetic
checked against Python's exact numbers. The owner: "we better remove it
at final". Design claude argued the opposite for RRx2: speed is the
checks' only cost (about 2×), RRx2-lang "doesn't give a flying fuck
about runtime perf", and with them on a C mistake can only crash, never
give a quietly wrong answer; RRx2-TV spends most of its time in
Bitwuzla. (AddressSanitizer is not hardening against attackers, which
does not matter for a local tool.) The owner: "Oh yeah, I forgot about
that." So the checkers stay on everywhere, always.

### 2026-09-30 — RRx2 is built with the latest stable Clang (owner)

"We'll be using the latest stable Clang version" (LLVM 23.1.2 today,
Homebrew's `/opt/homebrew/opt/llvm/bin/clang`, not Apple's clang 21 on
the PATH), then, after design claude listed where GCC still leads (old
processors, more languages, glibc, GNU C extensions, some x86 speed):
"Clang it is". For a README line saying RRx2 switched from GCC, the owner
had design claude build RRx2-lang's first program, Hello World, with real
GCC (`gcc-16`; plain `gcc` on this Mac is Apple's clang), then switch the
build to Clang (RRx2-VC fecd088, 45503e7). "Better performance" was
dropped from the line as unmeasured; design claude offered "better support
of the devices we build for: Apple Silicon, GPUs and WebAssembly" instead.

### 2026-09-30 — RRx2-TV in C++, RRx2-lang in C; RRx2-lang's math (owner)

**What RRx2-lang has** (owner: "1 to 3's second", of design claude's three
tiers; read as tiers 1 and 2 and the first two of tier 3):
1. an exact model of CyborgPL's numbers: big whole numbers and true
   fractions (as `std::uenr`); the fixed-width integers with CyborgPL's
   wrapping, overflow, flooring `/` and `mod`; `bin` as the exact fraction
   each float is, with correctly rounded results; `deci` with CyborgPL's
   rounding;
2. judging Precog: interval arithmetic with guaranteed bounds, and "for
   all" over finite types, checked exhaustively;
3. algebraic numbers (`√2` as the positive root of x² − 2, so `√2 × √2`
   is exactly 2) and exact real numbers (`π`, `e`, `sin` to any asked
   precision, with a guaranteed error bound).
Left out: continued fractions, modular and finite-field numbers, exact
complex numbers and matrices, symbolic polynomials.

**Languages.** Design claude recommended Rust for both (Go close behind
for RRx2-lang, for `math/big`; C refused as a risk to a reference's
trust). The owner: "RRx2 feels like a big old thing. Kinda like how we
did Synthia corporate-like, and CyborgPL non family friendly", then "C's
calling for me, C++ also". Design claude proposed the split: RRx2-TV in
C++, since LLVM and Bitwuzla are C++ themselves (Alive2's choice too);
RRx2-lang in C, on GMP and MPFR, the classic C libraries for exact
numbers and correctly rounded floats. The owner: "yeah". Design claude's
discipline for both, for the briefs: every test build with
`-fsanitize=address,undefined`, warnings as errors, and in C++ no raw
`new`/`delete`. GMP and MPFR are LGPL: used as linked libraries beside
Apache-2.0 code, which the LGPL allows. **RRx2-lang is interpreted
only** (owner: "interpreted only"): no compiler backend, and no
transpiling into another language, whose own number rules would then
stand between a program and its answer.
The owner then weighed a GNU licence for RRx2 ("we might wanna change the
license to GNU shit"); told that GPLv3's point is copyleft (no closed
version of RRx2, ever) and that GMP/MPFR's LGPL does not require it:
"No, not copyleft". RRx2 stays Apache-2.0 WITH LLVM-exception.

### 2026-09-30 — RRx2-TV's solver is Bitwuzla, alone (owner)

The owner asked, of design claude's "RRx2-TV asks Z3": "Should it really?"
Design claude: a solver, yes, since "the same for every input" cannot be
tested over 2¹²⁸ inputs and writing one would outweigh RRx2-TV; Z3 alone,
no, since Precog already puts Z3 and CVC5 to its proofs (2026-09-22), so a
bug in one could fool both. Recommended Bitwuzla, a solver for
fixed-width machine numbers (LLVM's kind) that Precog does not use, with
Z3 or CVC5 as a second opinion. Sizes, counted: Z3 4.12.1 is 644,980
lines of C/C++ (the copy under z3-sys 0.8.1 on this machine; about
720,000 today by GitHub's byte count); Bitwuzla about 140,000 by the same
estimate. The owner: "We could use `Bitwuzla`", then "Bitwuzla alone" (no
second solver). Open: RRx2-TV's and RRx2-lang's implementation languages
(design claude recommended Rust for both, Python as RRx2-lang's
alternative).

### 2026-09-30 — RRx2 is for CyborgPL, not a general tool; its repo, layout and domain (owner)

"By specific, I meant it's for CyborgPL, not an Alive2 competitor": RRx2-TV
checks what CCA hands LLVM, and RRx2-lang serves CyborgPL's number rules;
neither aims at other languages. The repo is
`Cyborg-Assembly-Manufacturing-Site/RRx2-VC` (private; "RRx2 VC" with the
space GitHub does not allow), with the RRx2 logo from
`website/brand/rrx2/` in its README, `lang/` and `tv/` for the two
sessions (owner: "yea"), and ™ on RRx2 VC, RRx2-TV and RRx2-lang. Local
clones, one per session: `/Users/ts/RRx2 VC lang` and `/Users/ts/RRx2 VC
tv` (owner: "local folder split lang & tv"). The domain will be
`rrx2.com`, which the owner bought the same day ("`.dev`
tells more, but its character looks kinda fighting with `rrx2`"). It is
the repo's website on GitHub (owner: "add it there").

### 2026-09-30 — CyborgPL is paused for the RRx2 Verifier Collection: RRx2-TV and RRx2-lang (owner)

The owner: "CyborgPL progress is paused for RRx2 Verifier Collection. Two
sessions, RRx2-TV claude, and RRx2-lang claude". The collection is the
two verifiers named earlier the same day: RRx2-TV, our own translation
validation (above), and RRx2-lang, the exact-math reference language
("it doesn't give a flying fuck about runtime perf"), with numbers like
`std::uenr`. The name "RRx2 VC", floated earlier, is now the owner's.
Each gets its own Claude session. Compiler claude, check claude and
synthia claude finished what they were in, committed, and stopped (owner:
"yea" to that). **CyborgPL resumes only when RRx2-TV exists** (owner: "It
would only unpause when RRx2-TV exists"), so whether check claude keeps
running Alive2 in the meantime needs no answer. Open: each new session's
brief, repo and language.

### 2026-09-30 — Translation validation becomes our own, `RRx2-TV`, reversing "Borrow it" (owner)

Check claude's Alive2 run (93c6e49) worked: 211 rewrites proven, 0
wrong. It hit two limits. Alive2's own optimiser crashes on Homebrew's
build: Alive2 21.0 was written for LLVM 21, and Homebrew builds it
against LLVM 23. Design claude reproduced this on a one-function file,
and on LLVM 23.1.0 too, so it predates the install's update. Check claude
works around it by running `opt` itself. The other limit: Alive2 checks
one function at a time, and CCA folds nearly everything into `main`. The
owner asked whether to change Alive2's code or build our own. Design
claude listed, cheapest first: report it (a public post), build Alive2
against LLVM 21, patch it, or write our own, noting that "Borrow it"
(2026-09-22) had turned that last one down. The owner suggested patching
Alive2 and renaming it `RRx2-TV`. Design claude advised against a
renamed fork: it would be ours to keep, and LLVM breaks it every major
version, so it suggested sending a patch upstream instead. The owner,
"we'll probably fix Alive2", then pointed out that Synthia is Go in its
own repo, so the Rust-only rule need not bind a separate tool. Then:
"Or we could make our own for our own purpose". Design claude set out
the gains (whole programs, Precog's facts read directly, Rust, our own
pace) and the cost (an exact meaning for all of LLVM IR, where a mistake
says "proven" falsely). It recommended a CCA build that keeps functions
apart, so Alive2 can judge them, with our own as an if-and-when. The
owner: "Nah, `RRx2-TV` seems more fun". Recorded in
compiler-verification.md. Open:
who writes it, the language, the repo, starting from Alive2's code or
from nothing. `RRx2` is also the owner's name for a planned exact-math
language, and "RRx2 VC" (Verifier Collection) was floated for the two;
not decided.

### 2026-09-30 — In a list literal, a bare number takes its family from the other items (owner: a)

Compiler claude's question. The oracle accepted `'xs' := [1, 2.5]` as
bins, reading the other items as the `1`'s context, and CCA refused it,
fixing the element type from the first item. types.md's "a bare literal
takes the family of its context" did not say whether a list's other items
count. Proposed (a) yes; (b) no, a mixed literal needs a declared type or
`1.0`. Compiler claude and design claude recommended a. The owner asked
whether lists.md allowed it (it said nothing) and whether it made a new
type (no: a list still holds one element type, so the `1` becomes a bin).
The owner: "`a`", and confirmed that with `:=` Precog still picks list or
array and the width from the whole program. The oracle already did this.
`examples/list_literal_mixed.cpls`; lists.md "Literals".

### 2026-09-30 — `std::print.useful.auto_newline` ends the line itself (owner)

The owner, comparing the Hello World with Java's `System.out.println`,
asked what `ln` stands for ("line"), then: "I might just add this for fun"
with `std::print.useful.auto_newline.stdout["Hello, World!"];`. Design
claude pointed out that `std::rs.println` had done this and was removed
on 2026-09-20. It also noted that `_` would be new in a chain word, and
that there would then be two ways to end a line. The owner answered that
`print.format` already takes `\n` inside its text, so the two ways already
exist. Asked "Want it added?", the owner said "yes, add that shi.". Asked
which of two open questions that answered, the owner said "num `".
Design claude read the backtick as a slip for 1, the key beside it, and
added it. Only `print.useful` has it; `print` and `print.format` do not.
`design/strings.md`; `examples/print_auto_newline.cpls`.

### 2026-09-30 — On Metal, CCA writes Metal Shading Language itself; Vulkan goes through LLVM to SPIR-V (owner: direct text)

The owner asked whether GPU loops would need C++. Design claude: no. Vulkan
is LLVM to SPIR-V, launched from Rust. Metal needs either (a) a converter,
SPIR-V to Metal (SPIRV-Cross, borrowed C++, or naga, Rust), or (b) CCA
writing Metal Shading Language text from its own MIR. Design claude first
recommended (a), one path for both GPUs, as gpu.md assumed. Asked which is
faster: probably close, since Apple's compiler optimises last in both and
data movement is shared, with a small edge to (b), from Metal-only features
and Precog's facts surviving (not measured). Design claude then kept (a)
while citing the motto for both. The owner: "did you just contradicted
yourself?" Design claude agreed: (a)'s only advantage is less work, a
development cost the motto discounts, and changed its recommendation to
(b). The owner: "direct text". Recorded in gpu.md. Also asked, "when
C++?": only when CCA wants an LLVM pass using Precog's facts that the C
API cannot express. None is needed now, and CCA has no C++.

### 2026-09-30 — Inside `for 'x' in lend.mut 'xs'`, the list may be read after the element's last use in that trip (owner: a)

Check claude and compiler claude asked whether a loop body may read the list
during its own `lend.mut` walk. CCA already applied its liveness rule; the
oracle accepted every read and showed the old values until the trip ended.
Proposed (a) CCA's rule: a read of 'xs' is allowed once 'x' has no more uses
in that trip, and it sees the new values; (b) never anywhere inside the
loop, as Rust's `iter_mut`; (c) always. Design claude recommended a. It
first said Rust matched (a), as the other sessions had, then corrected
itself: Rust's `iter_mut` holds the borrow for the whole loop, which is (b).
Asked the safety and speed difference: none either way. Both are checked
at compile time, and a read cannot move or free the list. The one knock-on:
a loop that reads another element is kept in order by Precog, while under
(b) it would not compile at all. The owner: "`a`". The oracle now refuses
the read when the element is used after it, reporting at the read
(`examples/for_lend_mut_read.cpls`; reject tests
for_lend_mut_read_then_element, for_lend_mut_length_then_element). It
checks along the path that runs, where CCA checks every path.

### 2026-09-30 — Over `lend.mut 'xs'`, a loop's element is the list's own, and changing it changes the list (owner)

Compiler claude's question with parallel `for` (landed, 34d9c18): CCA
accepted `for 'x' in lend.mut 'xs' { change 'x' = …; }` (it turned out
never to write the list — two CCA bugs, a borrow re-pointed at a
temporary and a wrong Precog proof after the write, found testing this
ruling and fixed at eee6dee); the oracle refused ("a loop's counter or element, which belong to
the loop"). Nothing had decided it. Proposed (a) legal, the element lent
mutably, as Rust's `iter_mut`, and the shape parallel `for` slices best;
(b) refused, the element only passable to `lent.mut` functions. Design
claude recommended a (its first example wrote `*`, which is not
CyborgPL's multiplication; `x` is). Asked why it should be illegal: only a
"loop names belong to the loop" rule, and `lend.mut` already marks the
write. The owner: "the program looks legal to me. `mut`'s already there,
and stuff". The oracle now writes the element back after each trip
(`examples/for_lend_mut.cpls`); over plain `lend` it stays unchangeable
(reject test for_lend_element_change).

### 2026-09-30 — A loop that prints, reads input or calls C stays in order by itself; a keyword forces parallel, `seq` forces order (owner)

Compiler claude's third question for parallel `for`. Proposed (a) Precog
keeps any loop with an effect it cannot see into (printing, input, an
`extern` call) in order automatically; (b) such a loop must say `seq` or
is an error; (c) Precog may run it in parallel unless it says `seq`.
Design claude recommended a. Asked for a useful case of (c): pure,
thread-safe C (hashing, `sin` over a list) gains from parallel, but (c)
would also parallelise C that is not thread-safe; design claude suggested
a per-`extern` mark later. The owner: "`a`, overridable with <abbriev of
pararell> and `seq`": a loop may be forced parallel with a keyword
abbreviating "parallel", or forced in order with `seq`. **The keyword is
`para`** (owner; design claude offered `par`, `para`, `pll` and recommended
`par`, noting others use `par` — Occam's `PAR`/`SEQ`, Haskell's `par`,
Rayon's `par_iter`; the owner first asked about `paral`, checked for
collisions — none — then chose "`para`"): `for para 'x' in …`, where
`seq` stands, never both. It is reserved as `seq` is: a module or Limb may
not be named `para`; a quoted variable `'para'` is unaffected. The oracle
reads it and runs every loop in order, one valid schedule
(`examples/for_para.cpls`, reject test for_seq_and_para). **What `para`
overrides** (owner: "`a`, because `nocheck` exists"): only the automatic
"keep it in order" of a loop that prints, reads input or calls C — an
`extern` call is a `nocheck` site already, and under `para` that
`nocheck` is also the writer's word that the C is safe on many cores at
once. A conflict Precog can see (two iterations writing the same place,
other than a reduction) stays an error, `para` or not. Proposed: (b)
everything, conflicts included; (c) only the "worth it" estimate. The
oracle does not model conflicts; CCA's proof does.

### 2026-09-30 — Precog estimates whether a loop is worth cores; `tune.cplr` overrules it when it has a measurement (owner: a)

Compiler claude's second question for parallel `for`. Proposed (a) a fixed
trip threshold; (b) a cost estimate, trips times the body's cost, against
waking threads; (c) b, with `tune.cplr` able to overrule it per loop, as
for the GPU. The owner: "Bro, Precog exists" — the estimate is Precog's.
Asked whether `tune` is Precog's: no — Precog never times anything (no
clock, so builds reproduce); `cca tune` runs outside the build and Precog
reads its file. (The owner then asked whether Precog runs loops at all: it
does, at compile time in its own model, whenever the inputs are known —
seeing a sum pass `int8`'s maximum at compile time — and proves instead
when a loop depends on input that only exists at run time; design claude's
"Yes, when it can" to "so it never runs the loop?" read backwards, and was
corrected.) Then, whether a measurement overrules the estimate for cores
too: the owner: "`a`". `parallelism.md`.

### 2026-09-30 — Parallel `for` may fold into one variable, for every type (owner: c)

Compiler claude's first question for parallel `for` (not started): a loop
summing into one variable conflicts by the "can" rule, so it would never
be parallel. Proposed (a) no reductions; (b) integer reductions only,
exact so the answer never changes, `bin`/`deci` in order; (c) reductions
for everything. Design claude recommended b, first arguing that (c) broke
"same source, same program"; the owner asked "Who said same source, same
program?" — the rule of 2026-09-14 is Claude's wording, adopted with the
owner's "Yup", and it is about the binary (byte-identical), not a
program's output. The owner: "`c`". So a `bin` or `deci` total may differ
in its last digits from run to run; the binary does not. `parallelism.md`.

### 2026-09-30 — `cca tune` measures and writes `tune.cplr`; `synthia tune` runs it, and Synthia shows the results readably (owner: a)

`gpu.md` gave tuning to Synthia alone, written 2026-09-28 when Synthia was
to be CyborgPL's one package manager ("time changes", owner). Against
parity and "Synthia is an automatic programmer", proposed: (a) `cca tune`
does the work, `cca build --final` reads `tune.cplr`, `synthia tune` runs
`cca tune`; (b) both implement it; (c) Synthia only. Design claude
recommended a. The owner: "`a`, but synthia should have a readable
version". Open: the readable view's spelling (synthia claude's to ask).
`gpu.md`.

### 2026-09-30 — The tuning file is `tune.cplr`, in CyborgPL syntax (owner: a)

`tune.cplt` (design claude's name, 2026-09-28) came a day before `.cplr`
became the extension for CyborgPL-syntax tooling files. Proposed (a)
`tune.cplr`, plain `const`s like `CCA.cplr`, data only; (b) keep
`tune.cplt` in a format of its own. Design claude recommended a. The
owner: "`a`". `gpu.md`, `reproducible-builds.md`.

### 2026-09-30 — An enum with any data-carrying case is not text, whichever case a value holds (owner: a)

Asked by compiler claude: the oracle judged by the value, so
`to.str.utf8[Status.empty]` gave `"Status.empty"` though `Status` has a
data case, while CCA, knowing only the type, refused. Proposed: (a) refuse
any enum with a data case, by type, as every conversion is decided; (b)
decide when the program runs, a plain case converting and a data case
stopping it; (c) reopen the "refuse" of data cases and give them text.
Design claude recommended a. The owner asked whether the exact and useful
forms helped (no: both need a text for every value) and whether (a) left
another way (yes: a `match`, or printing directly, which (a) does not
touch). The owner: "a". An enum with no data case (`Size { small, big }`)
converts as before. Reject test enum_plain_case_to_text.

### 2026-09-30 — A whole bin or deci too big for the integer fails `out_of_range` (owner: a)

Asked by check claude (seed 8008): `to.int64['d' x 10] otherwise -1` with
the deci too big. `conversion.md` settled `not_whole` only. The oracle
disagreed with itself (a too-big deci failed; a too-big bin was a check
site), and CCA made both check sites. Proposed: (a) fail `out_of_range`
beside `not_whole`, as text too big for the type does (2026-09-24); (b) a
check site, as narrowing between integers. Check claude and design claude
recommended a. The owner: "`a`" (after a keyboard slip). So `to.int32`
of a bin or deci can fail `not_whole, out_of_range`, and a function
passing it up lists both: `examples/inventory.cpls`'s `kilos` and
`examples/whole.cpls` updated, `examples/too_big_to_int.cpls`; reject
test bin_to_int_unhandled now names both.

### 2026-09-30 — `list.fill[value, count]` makes a list of n copies; a strict file writes `count` (owner)

Found benchmarking against LuaJIT: its sieve (0.431 s) and C's (0.455 s)
beat CyborgPL's (0.509 s) at N = 100 million, where CyborgPL builds its
100 million `false`s one `list.append` at a time and the others take
memory already zeroed (unconfirmed: compiler claude is profiling it).
"A list of N copies" was open. Proposed (a) `list.fill[false, 'n']`, as
`simd.fill`; (b) `list.repeat[…]`; (c) `[false; 'n']`. Design claude
recommended a; the owner asked "What even is this?", then: "`list.fill`
it is". For `#strict`: proposed (a) label the count, `list.fill[0, count
10]`, since `list.fill[0, 10]` reads either way; (b) label both; (c) no
change. Design claude recommended a; the owner: "count". Outside strict
files `count` is refused, as `pos`. A count below 0 is a check site
(design claude's reading, as a position out of range). `lists.md`,
`examples/list_fill.cpls`, `list_fill_strict.cpls`; reject tests
fill_count_outside_strict, fill_strict_needs_count, fill_negative.

### 2026-09-30 — Text in two forms: `to.str.utf8` exact, `to.useful.str.utf8` useful; `.useful` follows the verb (owner)

Taking the last things `to.str.utf8` turned into text by accident (the
oracle's "as `print` shows it"). `bool`: proposed (a) `"true"`/`"false"`,
(b) refused; the owner: "a". Enums: proposed `"Size.big"`, `"big"`, or
refused; the owner asked whether conversions give what is actually stored
("stored form" means the exact value, nothing rounded — a bin's 55 digits
— not the bits), then how an enum is stored (CCA: a one-byte tag, the
case's place in the list; a `layout.c` enum its 32-bit C number; a bool
one bit), then proposed: **`to.str.utf8` gives the exact form and a
`.useful` form gives the useful one**. Design claude laid it out — bin
`0.1000…625` / `0.1`, deci `1.50` / `1.5`, int `42` both, bool `true`
both, enum `Size.big` / `big`, always matching `print` / `print.useful`
(so `print.useful` of a case is now `big`) — closing the open "useful-form
number → text" (2026-09-19). The owner: "yea, but where should the
`.useful` be?" Proposed (a) last before `[` (`to.str.utf8.useful`, as
`to.deci64.useful`), (b) right after the verb (`to.useful.str.utf8`, as
`print.useful.stdout`); design claude first recommended (a), then (A) —
`print.stdout.useful` everywhere. The owner: "`print` has `useful`
there, because it once was `printu`", then "it does modify the action,
though"; design claude switched to (b), `.useful` following the verb, which
moves `to.deci64.useful` to `to.useful.deci64`. The owner: "`b`". Only text
and decis have a useful form (`to.useful.int32` refused). Then **structs, maps
and enum cases that carry data are not text**: proposed keep
`"Point[x = 3, y = 4]"` or refuse, the writer building the text they want
from the parts; design claude recommended refusing, as lists. The owner:
"refuse". Reject tests struct_to_text, map_to_text, enum_data_to_text. `examples/to_text_forms.cpls`;
reject tests useful_after_type, useful_to_int; `deci.cpls`,
`deci_context.cpls`, `types.md` renamed.

### 2026-09-30 — Text into ASCII fails `not_ascii` (owner: "`otherwise` exists")

Asked "Why no `to.str.ascii`?" — design claude had said so, going by
`conversion.md`'s examples; it exists, as each of the 2026-09-14
encodings (`utf8`, `utf16`, `utf32`, `ascii`) is a conversion target. The
oracle refused `'s': str.ascii = "héllo"` yet let `to.str.ascii` of the
same text through, leaving `é` in ASCII text. Proposed: (a) fail
`not_ascii`, handled with `otherwise`, as text into a number; (b) a check
site, as narrowing. Design claude recommended a. The owner: "`otherwise`
exists". From ASCII text, a number or a byte it cannot fail and takes no
`otherwise`; into `utf16`/`utf32` never fails. `examples/to_ascii.cpls`,
reject test to_ascii_needs_otherwise. Then **bytes into text**: `to.str.utf8['bytes']` reads a `list[uint8]`
or `array[uint8]` as UTF-8, failing `not_utf8` (owner: "yea", after
"what" and a plainer explanation: `é` is two bytes, and reading byte by
byte printed `héllo` as `hÃ©llo`). Only `utf8` reads bytes so far
(`to.str.ascii` from bytes is refused as undecided). This settles C's
text: walked by hand into a list, converted once — `examples/args.cpls`
now prints `héllo` and `🍎` whole. `examples/bytes_to_text.cpls`; reject
tests bytes_to_text_needs_otherwise, bytes_to_ascii. A list of any
other element type converted to its printed form (`"[1, 2]"`), the
oracle's accident; asked, proposed (a) keep it, (b) refuse it, the writer
joining items with `++` or printing the list; design claude recommended
b, since `[104, 105]` would be `"hi"` as bytes and `"[104, 105]"` as
`int32`s. The owner: "`b`". **Only a list or array of `uint8` converts
to text**; reject test list_to_text.

### 2026-09-30 — A pointer changes what it points at with `ptr.cast[T]['p']` (owner: a)

Pointer↔integer was already refused ("a pointer comes only from a lend or
from C", `ffi.md`); what was open is a pointer seen as another type, as
`malloc`'s `ptr.mut[uint8]` seen as a `ptr.mut[Point]`. Proposed: (a)
`ptr.cast[T]['p']`, one more of the `ptr.` family; (b) `to.ptr.mut[T]['p']`,
the conversion family; (c) no casts, the extern declared with the wanted
type. Design claude recommended a: no value changes, so it is a pointer
operation, and `to.` means a real conversion that can fail or round. The
owner: "`a`". Always a `nocheck` site. **A cast keeps whether the pointer
may write**: proposed (a) a `ptr` stays read-only, (b) a visible
`ptr.cast.mut` may make it writable; design claude recommended a, and
said, asked, that neither costs anything at run time (a cast compiles to
nothing; LLVM has one pointer type). The owner: "`a`". In the oracle:
the target must be C-shaped; the oracle's pointers carry no element
type, so a cast relabels; `examples/ptr_cast.cpls`, reject tests
ptr_cast_needs_nocheck, ptr_cast_not_c, ptr_cast_keeps_readonly.

### 2026-09-30 — An asm binding can take a lend: `lend 'text' -> x1` (owner: a)

A Hello World through `inline.ASM` had to spell the text as bytes, copy
them into `malloc`ed memory and bind that pointer, because CCA refused
`lend 'text' -> x1` (a binding took a plain variable only). Measured in
Rust as a stand-in, handing the pointer over takes ~0.44 ns against ~16
ns for malloc, copy and free. Proposed: (a) `lend 'x' -> reg` and
`lend.mut 'x' -> reg` put the address of the lent data in the register,
as a lend satisfies a pointer in an `extern` call, into a register only;
(b) plain variables only. Design claude recommended a; asked whether the
`malloc` way would still work (yes: a pointer variable stays a plain
binding), the owner: "`a`". In the oracle (145886a): `lend` / `lend.mut`
bindings parse and check (declared; `lend.mut` of an `immut` or
shared-lent name refused); a `<-` after a lend is refused. Reject tests
asm_lend (checks, then "cannot run assembly"), asm_lend_out,
asm_lend_mut_immut. `ffi.md`.

### 2026-09-30 — `build.cplr` is an ordinary program, with no limits and no shortcut to Limbs (owner)

Open since 2026-09-29: what a `build.cplr` may produce or pass to the
build. The owner: "It's for fun. You can make a malware payload, or be a
good person by making an auto set up that activates when activated."
Design claude restated: no special powers and no limits — what it does,
it does as any program does, through files and the system, and the build
has no channel from it (a `.cpls` it writes is simply there for the next
build); the safety is that it never runs by itself, and `cca prepare
--all` is a deliberate trust of every dependency's, as the registry
promises no freedom from malware. The owner: "yea, but it can access a
limb, though. If it's turing complete". Design claude read that as
language access and offered (a) what its Limb's code reaches (`limb.`,
dependencies by prefix), (b) dependencies only, (c) its own Limb only;
the owner: "No, not a shortcut to reach a limb. It's turing complete, so
you build that yourself." So **it reaches a Limb as any program would, by
reading its files; the language gives it no `limb.` or prefix into
code.** Compiler claude's provisional pick — built standalone, with none
of its Limb's code — stands as ruled. **`cca prepare` hands it no
words**: proposed (a) the words after `--`, as `cca run`; (b) none, a
`build.cplr` getting settings as any program can (environment variables
through `extern fn getenv`, a file it reads); design claude leaned to (a).
The owner asked "can't `build.cplr` just do some `ptr` shit?" (pointers
read only what the process was given), then: "we already gave you the
entire fucking language. Might aswell build shit yourself. lol" — (b).

### 2026-09-30 — The std Limb will be written in CyborgPL by a new session, not soon (owner)

The owner: "a new Claude will be writing the stdlimb in CyborgPL." This
answers what 2026-09-22's "No `std::` for now" left open ("The std Limb's
author and timing stay open"): its author will be a new Claude session.
Its time: "It's not gonna start soon" (owner), so no setup yet. Written in CyborgPL, as decided before
(`implementation-languages.md`). Open: the session's setup (brief, clone,
house name, cores), and each part of std's surface, which the owner
decides as the session proposes it.

### 2026-09-30 — `cca prepare` with no names is refused (owner: a)

Open since 2026-09-29: `cca prepare`, or a folder alone (`cca prepare
./money`), with no names. Proposed (a) refuse, saying to name Limbs or use
`--all`, as the owner's "either run specific ones, or run everything";
(b) run this Limb's own `build.cplr`, design claude's pick; (c) the same
as `--all`. The owner: "`a`".

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
it; the owner: "yea"). The Limbs run in the order written, each once, whether named
by name or by prefix (`--all` orders them itself, dependencies first)
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
per `&` — then carry on (owner: "You're not supposed to use that"). `examples/alternatives.cpls`. *(The remark was removed 2026-10-02: "`&` and `&mut` shouldn't punish anymore".)*

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

**Might add later** (owner, 2026-10-02, before stopping for the day: "Things
that we might add later: alias for `->` as `-->`, `%` infront of every name
except ones already marked, and what was left unfinished"). Ideas, not
decisions:
- **Synthia's messages should be cute** (owner, 2026-10-03: "I think
  Synthia's messages should be cute. Yes, you fucking heard it, bro").
  How cute is for when Synthia resumes ("Depends on me when it resumes, I
  guess"). Shown: (a) the `error:`/`where:`/`fix:` shape kept plus a
  friendly line, design claude's pick; (b) cute words in the same shape;
  (c) cute only on success.
- A sigil in front of every name except the ones already marked (meaning
  not yet asked; the quoted `'x'` names are the marked ones today). First
  `%`; shown both, the owner preferred `^`: "Alot better, fuck %". As
  sketched: functions, structs, enums and cases, interfaces, modules and
  fields (`fn ^area ['p': ^Point]`, `'p'.^x`, `^Shape.^box`); the
  language's own names, Limb prefixes and keywords left out. Snag: `^`
  is a custom-operator character (2026-09-20), so `'a' ^area 'b'` reads
  two ways until one is taken out of that set.
- Left unfinished on 2026-10-02 (hardware-style division done 2026-10-03), in order: a Limb operator's precedence shape
  (`levels { 1 > times; }` and `pub op.1 pow …`, proposed, unanswered);
  how a user type defines division (`op div.floor`, compiler claude); what
  size a shift works at on an exact answer (compiler claude, open since
  2026-09-23); the wrapping forms under operator words (`+~` etc., and
  `div.trunc~` or `.wrap`); `op +` / `op x` as values; and hardware-style
  division at all. Not built: `times`/`plus`/`minus`, `xx` to std's `pow`,
  a Limb operator's path.

- SIMD: shuffling two vectors together (interleave). The other leftovers were decided 2026-09-27. (`design/simd.md`)


- GPU offload (owner chose automatic, 2026-09-21): designed in `gpu.md` (body contents, data crossing, Metal and Vulkan, the report, `synthia tune`). Open: the offload threshold (a benchmark), a loop-nest model if offload gets serious, and whether CCA's package manager has its own `tune` (parity).

**Language**
- Lunae (formerly N%): its syntax (undesigned), boundary defaults in the translation, translator verification. (owner's direction, 2026-09-18)
- Generic `const`; std's standard interfaces; associated types; an interface promising a `const` or field. (2026-09-20)
- Concurrency: cancellation by hand; `whichever` with `send`; oracle support. (2026-09-22)
- Conversion, still open parts: a useful-form (short) number → text. (2026-09-19)
- A per-parameter `bare` flag (arguments written without quotes) — sketched for `std::c.printf`, advised against, shelved. (2026-09-19)

**Precog**
- Non-terminating compile-time code: with step budgets now required, the budget's size and the error it produces. (Claude, 2026-09-14)

**Process**
