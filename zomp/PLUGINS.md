# Plugins: how plugins plug into Zomp (proposal)

**A core is a plugin since 2026-10-09** (owner: "Plugin next", then "yea, go"): a strike lists `plugins = [ricochet::Plugin[…]]`. The crates are under `plugins/` since the Zomp rename (2026-10-09).

**Since 2026-10-07 (owner; DECISIONS 2026-10-07, "No base and no
codextras")**: there is no base below the language and no codextras.
Ground is the lowest level and does what this proposal gave the base
(reading files and manifests, loading plugins, checking powers, handing IR
to LLVM); dialects were tried and dropped the same day ("just let them
fork so they could do whatever they want"). A plugin may also **take powers
away** (owner: "`a`"; a game engine's plugin forbidding raw memory to the
limbs built with it). So the three boundaries below are one, Ground ↔
plugin; (A) and (C) are inside Ground now, and the `zomp-base` crate from
step 1 is Ground's power ledger, not a layer under it. The rename of
Ground to "base CyborgPL" is recorded and not applied ("Record, don't
implement yet"). MTM is gone (its spellings are old ones, refused); the
slimmer base added TAS, AI, WSMW, Money, UniTect and CAPS, and ASP took
the range types and length bounds (section 1). **ASP and Genny are
retired** (owner, 2026-10-08; built 2026-10-09): what they held is the
Ground's, Zag's base, so a manifest listing either is refused.
**Plugins go by expertise since 2026-10-12** (owner): IEG and TAS are the
Ground's too, and WSTN, UniTect and CAPS are one plugin, unicode; CNC holds
everything about C (2026-10-10). The text below is the
proposal as answered, with "the base" and "the codextra" read as
Ground.

Compiler claude's proposal, 2026-10-07, for the owner to approve
(2026-10-06: "compiler claude proposes the plugin interface, with the
owner approving"). **The owner answered all six questions the same day**
(DECISIONS 2026-10-07, "The plugin interface"); each answer is under its
question, marked **Decided**, and where it differs from the recommendation
the text below it describes the decided way. **Nothing here is built**;
starting is the owner's call. Where there is a real
choice, it is a numbered question (**Q1**, **Q2**, …) with alternatives
and a recommendation; those are the owner's. Where the answer follows
from what is already decided, or is plain engineering, it is marked
**(engineering)**, and compiler claude settles it unless the owner
wants a say. Two gaps that block parts of this are at the end; they are
the owner's, and nothing here fills them in.

Fixed points this builds on (`design/cores.md`; DECISIONS 2026-10-05 to
2026-10-07): the base reads files, loads the codextra and plugins, enforces
grants, and hands LLVM IR to LLVM unread; the codextra (Ground) does
the syntax, types, borrows, the prover and the IR; the codextra limits what
plugins may do; plugins grant powers, add syntax and depend on plugins; every
limb lists the powers it uses; a program may only `deny`; a lone file has
every plugin and power; plugins are named by registry ID and version.

## 1. Where Zomp is today

Each official plugin is a switch inside one compiler. Listing MTM lets the lexer's alternative spellings through; listing MCPS
lets `concur` and `for para` parse; and so on. Concretely, today:

| Plugin | Where it lives in Zomp now |
|---|---|
| MCPS | `concur`, `whichever`, `channel`, `for para` in the gate; `par.rs`, `codegen/par.rs`, `codegen/tasks.rs` |
| unicode | a parser pre-scan refusing a name past ASCII (was WSTN); `str.utf16`, `char.grapheme`, counting and `case.any` in the gate (was UniTect and CAPS) |
| CNC | `extern.C`, `incl.C`, `layout.C` and the `C.` types in the gate; `cheader.rs`, the `C.` types in `target.rs` |
| AI, WSMW, Money | words in the gate (`brain16`, `int128`, `not_std.`, `deci64`, …); their meaning still all of `check/` |
| IEG, TAS | retired (2026-10-12): `no_check`, the marking rule in every strike, `on.machine`, `test fn` and `must` are the Ground's, which grants `vouch` and the operator powers to every strike |
| std | not a plugin: a limb Zomp does not have yet (`std::` is a stub) |
| MTM | gone (2026-10-07): its spellings are old ones, refused by name in every limb |
| ASP, Genny | retired (2026-10-08): `to.`, `check`, true-value literals, range types, generics and interfaces are the Ground's; their IDs are refused, naming why |

So "moving the plugins out" is two different jobs: the thin ones (WSTN,
the gate words, the syntax half of MCPS) are already almost separate; the deep ones
(IEG, and ASP and Genny until they were retired) are woven through the type checker, the prover and the IR,
because they change what a number or a type *means*. The proposal treats
them differently on purpose.

## 2. The three interfaces

There are three boundaries, not one:

```
base  <──(A)──>  codextra (Ground)  <──(B)──>  plugins
  └────────────────────(C)─────────────────────┘
```

- **(A) base ↔ codextra**: small and stable. The base gives the codextra
  the files, the manifests and the loaded plugins; the codextra gives back
  LLVM IR, a list of power uses, and messages.
- **(B) codextra ↔ plugin**: the big one, and **the codextra's to define**,
  since "a codextra defines which powers exist and limits plugins". A plugin
  written for Ground is a Ground plugin; another codextra defines its
  own (B). So (B) below is *Ground's* plugin interface.
- **(C) base ↔ plugin**: only loading, identity, dependencies and the power
  catalogue, never syntax or IR. The base enforces grants, so it must
  know which powers each plugin adds and grants, without understanding
  them.

### (A) What a codextra provides (engineering, in outline)

```rust
pub trait Codextra {
    fn id(&self) -> RegistryId;                // and version
    fn read_manifest(&self, text: &str) -> Result<Manifest, Message>;
    fn powers(&self) -> PowerCatalogue;         // the powers that exist
    fn defaults(&self) -> Vec<PowerGrant>;      // e.g. c_calls to every limb
    fn build(&self, job: &Job, plugins: &[&dyn Plugin]) -> Result<Built, Vec<Message>>;
}

pub struct Built {
    pub ir: LlvmModule,          // textual IR or bitcode, handed on unread
    pub uses: Vec<PowerUse>,     // (limb, power path, thing, where)
    pub messages: Vec<Message>,  // warnings and notes
}
```

The base checks `uses` against the manifests (section 4) before handing
the IR to LLVM, so a codextra cannot use a power nobody granted by
forgetting to check: the base does not trust it to. `read_manifest`
is the codextra's because the base does not know what a quoted name is
(cores.md: Ground reads `Ricochet.zv`).

### (B) What a Ground plugin provides

A plugin is two parts, as cores.md has it: **Rust** (what the compiler runs)
and **CyborgPL** (code it brings, compiled as a limb of its own). The Rust
part answers four kinds of question:

1. **Which syntax it adds**: entries in Ground's hook menu (section 3).
2. **What its syntax means**: each hook's node lowers to Ground code,
   to calls into the plugin's own CyborgPL part, or to IR (section 5).
3. **Which powers it adds and grants**: its catalogue entries, and its
   grants (to every limb, or to favourites by registry ID).
4. **Contracts** for the prover: what each operation it adds promises and
   requires, so the prover can reason through it (section 5).

```rust
pub trait Plugin {
    fn id(&self) -> RegistryId;
    fn needs(&self) -> Vec<CoreRef>;            // MCPS needs IEG
    fn powers(&self) -> PowerCatalogue;         // added, never lifted
    fn grants(&self) -> Vec<PowerGrant>;        // to all, or to IDs
    fn hooks(&self) -> Vec<HookClaim>;          // which syntax it claims
    fn parse(&self, hook: HookId, tokens: TokenSlice) -> Result<Node, Message>;
    fn meaning(&self, node: &Node, cx: &TypeCx) -> Result<Meaning, Message>;
}
```

## 3. Adding syntax: the hard part

**Q1. How a plugin adds syntax to Ground's parser.**

- **(a) A fixed menu of hooks.** Ground names the places a plugin may
  extend, and a plugin claims entries. The menu, read off what the official
  plugins already need:

  | Hook | Example today |
  |---|---|
  | a statement word | `concur …`, `whichever …`, `check …` |
  | a prefix head, `head.…[…]` | `to.int8[…]`, `to.int8.bits[…]` |
  | a narrowing after an operator | `.wrap`, `.masked`, `.ieee` |
  | a word operator, with a level | `mean`, std's `plus` |
  | a prefix operator | std's `-`, `!` |
  | a type head | `!std.int24`, `channel[…]` |
  | a modifier on `for` | `para`, `seq` |
  | a marker before a statement | `!check`, `NO_FUCKING_CHECKS` |
  | an item kind | `interface`, `satisfies` |
  | an `inline.` language | `inline.LLVM`, `inline.MSL` |
  | a spelling of an existing token | MTM's `->`, `immut`, `i32` |
  | the characters a name may hold | WSTN |

  Ground keeps precedence, spans and error messages; two plugins claiming
  the same entry are refused when the manifest is read, naming both.
  New *kinds* of syntax need a new entry in Ground.

- **(b) Token rewriting**, like Rust's procedural macros: a plugin is given
  the tokens after its word, up to a matching close, and gives back plain
  Ground tokens (calling into its CyborgPL part). Any shape fits, but
  messages point at rewritten code, the prover sees only the expansion, and
  two plugins' rewrites can interfere in ways neither checks.

- **(c) Grammar fragments**: each plugin ships grammar rules that Ground
  merges into its parser. The most expressive, but Ground's parser is
  hand-written recursive descent and would be rewritten as a generated
  one; ambiguity between two plugins' grammars is found only by the
  generator, if at all; and errors get worse.

**Decided: (a), with (b) inside the statement-word and prefix-head hooks
only** (owner: "yea"); two plugins claiming one entry are refused when the
manifest is read. **Recommendation, as it was: (a), with (b) inside the statement-word and prefix-head
hooks only.** Every syntax the official plugins have fits (a); a hook's
parse may take a balanced run of tokens raw, which covers the odd shape
without opening the whole grammar. Conflicts are caught at load, where a
user can act on them.

**Q2. Whether a plugin's words may shadow a limb's names.** Today `on` and
`my` became keywords, and enum cases named `on` were renamed. With
third-party plugins, listing one could break code that used its word as a
name. (a) A plugin's words are reserved only in limbs that list it
(recommended: listing is the opt-in, and the message names the plugin);
(b) a plugin's words never reserve anything, and are found by position;
(c) as today, a word becomes a keyword everywhere.

**Decided: (a) and (b), the plugin choosing** (owner: "`a`, `b` depends on
the plugin"): a plugin's words are reserved only in limbs that list it, and a
plugin may instead leave a word unreserved, found by its position.

## 4. Powers: how grants flow and are checked

All engineering, given the rulings, except where marked:

1. **The catalogue.** Ground's `powers()` gives the tree it defines
   (`inline_assembly`, `c_calls.functions`, `raw_memory.read`, the four
   extension powers, …). Each listed plugin's `powers()` adds entries under
   the paths it may add to. Ground decides where a plugin may add; a plugin
   cannot add above or beside a Ground power it is not allowed to
   extend, and nothing lifts Ground's limits (borrows, the prover).
2. **Grants.** Ground grants its defaults to every limb (the three
   dangerous powers). Each plugin grants to every limb built with it, and
   may grant more to favourites named by registry ID. **A plugin may also
   take a power away** from every limb built with it (owner, 2026-10-07:
   "`a`"): a taken power is not granted, whoever else grants it, and the
   message names the plugin that took it.
3. **Listing.** Each limb's manifest lists the powers it uses
   (`powers = [ricochet::Power.c_calls,]`); the program's own limb too.
4. **Using.** The codextra reports every use as `(limb, power path,
   thing, span)`: `c_calls` with thing `"sqrt"`, `vouch` at a `!check`.
5. **The base's check** of each use, in this order, each with its own
   message: granted (by Ground or a listed plugin)? listed by the limb?
   not denied by the program (`deny`, with `.except[…]` and `.for[…]`)?
   A lone file skips 1 to 3: it has every plugin and power.
6. **The record.** The build writes the list of uses per limb, which the
   registry shows and which an upgrade compares ("this version starts
   using `c_calls`").

**Q3. Whether a third-party plugin's contracts are trusted by the prover.** A
plugin's operation comes with a contract ("returns at most its first
argument"). If the prover trusts it, every proof downstream depends on that
plugin's honesty. (a) Trusted, the plugin being trusted already by the
warning's terms; (b) trusted only for verified plugins, an unverified plugin's
operations being opaque calls whose results the prover knows nothing about
(recommended: proofs then never rest on unverified code, at the price of
more run-time checks where such a plugin is used); (c) trusted, and every
site resting on one reported in the build's record.

**Decided: (a), every plugin's contracts trusted, official or not** (owner:
"Holy registry lock in" of trusting only verified ones, and on trust by an
RRx2 proof, "they're just gonna fork our entire project instead"; then
"`a`"). the prover treats a listed plugin's contracts as true, as it does
IEG's; trusting a plugin is listing it, as the approved warning says.

## 5. Meaning and IR

**(engineering)** A plugin's node lowers in one of three ways, chosen per
node:

1. **To Ground code**: the node is a call to a function in the plugin's
   CyborgPL part, or plain Ground it builds. Ground then types it,
   checks borrows, runs the prover and makes IR as for any code. This is the
   default, and the one every third-party plugin should reach for.
2. **To IR with a contract**: the plugin gives an LLVM IR template, and a
   contract the prover uses (Q3). This is IEG's way (`inline.LLVM`, the raw
   operations), and the reason RRx2 checks IEG's IR against its
   contracts.
3. **To runtime**: a call into a static library or bitcode the plugin ships
   (MCPS's thread pool, Metal's command queues).

What a plugin may not do: replace Ground's types, borrow rules or the prover,
or reach into another plugin's nodes. A plugin that needs another (MCPS needs
IEG) calls it through its CyborgPL part or its declared nodes.

**The deep official plugins.** ASP changes what a literal and plain maths
*are*; Genny adds type variables to the type checker; IEG decides what
needs a mark. None of that is a node to lower.

**Q4. How the deep official plugins move out.**

- **(a) Policies Ground owns, switched on by the plugin.** ASP's,
  Genny's and IEG's rules stay in Ground's code; the ASP plugin is the
  switch, plus its syntax hooks. The plugin interface is honest about
  this: Ground defines a few *policies* (number meaning, generics, the
  marking rule) that only its official plugins may enable.
- **(b) Real separation**: Ground exposes its type checker and the prover
  through the interface, deep enough for ASP and Genny to be written
  outside it. Anyone could then write a different ASP. It is a far larger
  and far less stable interface: every change to the checker is a change
  to the plugin API.
- **(c) Some of each**: (a) now, and a policy moves to (b) the day a
  second author wants to write one.

**Decided: (b), real separation** (owner: "`b`"): Ground opens its type
checker and the prover far enough that ASP, Genny and IEG are written outside
it, and anyone could write an ASP2. The cost is accepted: a large
interface, where a change to the checker is a change to the plugin API.
(The recommendation had been (c), starting as (a).) Step 6 below is this
separation.

## 6. Finding, loading and versioning plugins

**Q5. What a plugin is, physically.**

- **(a) A native library (`.dylib`, `.so`, `.dll`) behind a C ABI.** Rust
  has no stable ABI between compiler versions, so the boundary is `extern
  "C"` functions and `#[repr(C)]` data (or a crate like `abi_stable`),
  versioned by an interface number. The fastest, and a plugin can do
  anything the compiler can, including crash it. One build per target
  (macOS ARM, Linux x86, …).
- **(b) A WebAssembly component** (the WebAssembly component model, run by
  `wasmtime`). A plugin is written in Rust and compiled to `wasm32-wasip2`,
  so "Rust plugin" still holds. One file runs on every machine, the plugin
  runs in a sandbox (it cannot read files or the network unless the base
  hands it something), and the interface is described in a typed IDL
  (WIT) that versions cleanly. Slower per call, which matters for a parse
  hook called per token, not for one called per construct; and `wasmtime`
  adds weight to CCA.
- **(c) A separate process** spoken to over a pipe (as an editor speaks to
  a language server). Any language, and a crash cannot take the compiler
  down; the slowest, and awkward for fine-grained hooks.

**Decided: (a), native libraries behind a stable C ABI, for every plugin;
no WebAssembly sandbox** (owner: "the OS lib thingies file type"; asking,
on the way, why the language was getting restricted). A plugin is trusted
when it is listed, as the approved warning says. The official plugins are
still built into Zomp, and Ground into Zomp as the default codextra (not
asked; as proposed). **Recommendation, as it was: (b) for every plugin that is not official; the official
plugins built into Zomp as Rust crates behind the same trait.** The registry
then ships one file per plugin version, a stranger's plugin cannot read your
disk while it compiles your code, and the official plugins pay nothing. The
warning stays exactly as approved: a sandboxed plugin can still change what
your program does.

**(engineering)**, given any of the three:

- **Where they live.** Installed plugins in the machine-wide folder beside
  `installed.cplr`: `…/CyborgPL/plugins/<id>/<version>/`, holding the plugin
  file, its CyborgPL part, and a `Plugin.cplr` manifest (name, ID, version,
  the interface version it was built for, needs, powers it adds). A
  `ricochet::Plugin[path = …]` points at such a folder. Synthia downloads into
  it; Zomp never downloads (as for std).
- **Two versions.** The plugin's own version (which release, the date and
  time) and the interface version (which (B) it speaks). Zomp loads a plugin
  whose interface version it supports, and otherwise names both.
- **Pinning.** A lockfile beside `Ricochet.zv` records each plugin's content
  hash, so the same manifest always loads the same bytes.
- **Verified plugins** are the official ones, built into Zomp and checked as
  Zomp is (RRx2 for IEG's IR). Any other plugin gets the approved warning.

**Q6. Whether an unverified plugin may run at all in a build with
`--assert-proofs`, or in the registry's own build of a published limb.**
(a) Yes, with the warning, as everywhere; (b) refused there, those builds
being where trust is checked (recommended: a published limb's build on
the registry then never runs strangers' code).

**Decided: (a), an unofficial plugin runs there too**, and the registry
builds inside a throwaway, sealed machine, so a native plugin cannot harm
its servers (owner: "yea", to design claude's suggestion).

## 7. What changes in Zomp, and in what order

Each step leaves Zomp working and the suite green (engineering, unless a
question above changes it):

1. **Split the crate**: `zomp-base` (files, manifests, loading, the power
   check, the LLVM handoff), `signatar` (the rest of today's compiler),
   and `plugins/*` (one crate per official plugin, empty at first). No
   behaviour changes.
2. **The power ledger**, before any plugin: Ground reports uses, the
   base checks them against `powers` and `deny`. This is the powers
   ruling built, and needs nothing below.
3. **The hook menu** (Q1) inside Ground, with today's switches rewired
   through it: the gate tables become claims made by `plugins/asp`,
   `plugins/mcps`, and so on.
4. **Move WSTN out** first (only name characters), then the slimmer
   base's word-only plugins (TAS, AI, WSMW, Money, UniTect, CAPS): the
   smallest plugins, proving the menu. (MTM was to be first; it is gone.)
5. **MCPS's syntax and runtime**, then the **GPU plugins**, which are mostly
   new syntax lowering to runtime calls (section 5, way 3).
6. **IEG** as Q4 decides (ASP and Genny are retired: the Ground's).
7. **External loading** (Q5: native libraries, C ABI) last, once the official plugins have used the
   interface long enough to stop it moving; then the warning, the
   lockfile, and `ricochet::Plugin[path = …]` working.

## 8. Gaps that block parts of this (the owner's)

1. **Which powers MCPS, AutoGPU / Metal / Nvidia / Vulkan, WSTN and the
   new plugins add, and which a plugin may take away.** Section 4 can be built without them, but each plugin's
   `powers()` cannot be written, and step 5 cannot start, until it is
   known whether (say) spawning threads, a GPU queue, or a name in another
   script is a power a program may deny.
2. **IEG's raw operations** (`add`, `negate`, `compare` are design
   claude's sketch). Their names, which types they take, and their
   contracts decide IEG's nodes and the IR templates RRx2 checks; IEG
   cannot move out (step 6) before them.

(The question of loading another codextra is gone with codextras,
2026-10-07: a deeper change than a plugin can make is a fork.)
