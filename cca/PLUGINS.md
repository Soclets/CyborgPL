# Plugins: how cores plug into CCA (proposal)

**Since 2026-10-07 (owner; DECISIONS 2026-10-07, "No base and no
codextras")**: there is no base below the language and no codextras.
Signatar is the lowest level and does what this proposal gave the base
(reading files and manifests, loading cores, checking powers, handing IR
to LLVM); dialects were tried and dropped the same day ("just let them
fork so they could do whatever they want"). A core may also **take powers
away** (owner: "`a`"; a game engine's core forbidding raw memory to the
limbs built with it). So the three boundaries below are one, Signatar ↔
core; (A) and (C) are inside Signatar now, and the `cca-base` crate from
step 1 is Signatar's power ledger, not a layer under it. The rename of
Signatar to "base CyborgPL" is recorded and not applied ("Record, don't
implement yet"). MTM is gone (its spellings are old ones, refused); the
slimmer base added TAS, AI, WSMW, Money, UniTect and CAPS, and ASP took
the range types and length bounds (section 1). The text below is the
proposal as answered, with "the base" and "the codextra" read as
Signatar.

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
2026-10-07): the base reads files, loads the codextra and cores, enforces
grants, and hands LLVM IR to LLVM unread; the codextra (Signatar) does
the syntax, types, borrows, Precog and the IR; the codextra limits what
cores may do; cores grant powers, add syntax and depend on cores; every
limb lists the powers it uses; a program may only `deny`; a lone file has
every core and power; cores are named by registry ID and version.

## 1. Where CCA is today

Each official core is a switch inside one compiler. Listing ASP lets the
parser accept `to.` and makes literals true values in the checker;
listing MTM lets the lexer's alternative spellings through; listing MCPS
lets `concur` and `for para` parse; and so on. Concretely, today:

| Core | Where it lives in CCA now |
|---|---|
| ASP | `to.` and `check` in the parser's word gate; literal meaning in `check/expr.rs` (`raw_files`); `to.T.bits`; range types and length bounds (the type parser) |
| IEG | `no_check` / `NO_FUCKING_CHECKS` in the gate; the marking rule (`needs_mark`); `on.machine` |
| Genny | generics and `interface` in the gate, then all of `check/` |
| MCPS | `concur`, `whichever`, `channel`, `for para` in the gate; `par.rs`, `codegen/par.rs`, `codegen/tasks.rs` |
| WSTN | a parser pre-scan refusing a name past ASCII |
| TAS, AI, WSMW, Money, UniTect, CAPS | words in the gate (`test fn`, `must`, `brain16`, `int128`, `not_std.`, `deci64`, `char.grapheme`, `case.insensitive`, …); their meaning still all of `check/` |
| stdkit | not a core: a limb CCA does not have yet (`stdkit::` is a stub) |
| MTM | gone (2026-10-07): its spellings are old ones, refused by name in every limb |

So "moving the cores out" is two different jobs: the thin ones (WSTN,
the gate words, the syntax half of MCPS) are already almost separate; the deep ones
(ASP, IEG, Genny) are woven through the type checker, Precog and the IR,
because they change what a number or a type *means*. The proposal treats
them differently on purpose.

## 2. The three interfaces

There are three boundaries, not one:

```
base  <──(A)──>  codextra (Signatar)  <──(B)──>  cores
  └────────────────────(C)─────────────────────┘
```

- **(A) base ↔ codextra**: small and stable. The base gives the codextra
  the files, the manifests and the loaded cores; the codextra gives back
  LLVM IR, a list of power uses, and messages.
- **(B) codextra ↔ core**: the big one, and **the codextra's to define**,
  since "a codextra defines which powers exist and limits cores". A core
  written for Signatar is a Signatar core; another codextra defines its
  own (B). So (B) below is *Signatar's* core interface.
- **(C) base ↔ core**: only loading, identity, dependencies and the power
  catalogue, never syntax or IR. The base enforces grants, so it must
  know which powers each core adds and grants, without understanding
  them.

### (A) What a codextra provides (engineering, in outline)

```rust
pub trait Codextra {
    fn id(&self) -> RegistryId;                // and version
    fn read_manifest(&self, text: &str) -> Result<Manifest, Message>;
    fn powers(&self) -> PowerCatalogue;         // the powers that exist
    fn defaults(&self) -> Vec<PowerGrant>;      // e.g. c_calls to every limb
    fn build(&self, job: &Job, cores: &[&dyn Core]) -> Result<Built, Vec<Message>>;
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
(cores.md: Signatar reads `CCA.cplr`).

### (B) What a Signatar core provides

A core is two parts, as cores.md has it: **Rust** (what the compiler runs)
and **CyborgPL** (code it brings, compiled as a limb of its own). The Rust
part answers four kinds of question:

1. **Which syntax it adds**: entries in Signatar's hook menu (section 3).
2. **What its syntax means**: each hook's node lowers to Signatar code,
   to calls into the core's own CyborgPL part, or to IR (section 5).
3. **Which powers it adds and grants**: its catalogue entries, and its
   grants (to every limb, or to favourites by registry ID).
4. **Contracts** for Precog: what each operation it adds promises and
   requires, so Precog can reason through it (section 5).

```rust
pub trait Core {
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

**Q1. How a core adds syntax to Signatar's parser.**

- **(a) A fixed menu of hooks.** Signatar names the places a core may
  extend, and a core claims entries. The menu, read off what the official
  cores already need:

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

  Signatar keeps precedence, spans and error messages; two cores claiming
  the same entry are refused when the manifest is read, naming both.
  New *kinds* of syntax need a new entry in Signatar.

- **(b) Token rewriting**, like Rust's procedural macros: a core is given
  the tokens after its word, up to a matching close, and gives back plain
  Signatar tokens (calling into its CyborgPL part). Any shape fits, but
  messages point at rewritten code, Precog sees only the expansion, and
  two cores' rewrites can interfere in ways neither checks.

- **(c) Grammar fragments**: each core ships grammar rules that Signatar
  merges into its parser. The most expressive, but Signatar's parser is
  hand-written recursive descent and would be rewritten as a generated
  one; ambiguity between two cores' grammars is found only by the
  generator, if at all; and errors get worse.

**Decided: (a), with (b) inside the statement-word and prefix-head hooks
only** (owner: "yea"); two cores claiming one entry are refused when the
manifest is read. **Recommendation, as it was: (a), with (b) inside the statement-word and prefix-head
hooks only.** Every syntax the official cores have fits (a); a hook's
parse may take a balanced run of tokens raw, which covers the odd shape
without opening the whole grammar. Conflicts are caught at load, where a
user can act on them.

**Q2. Whether a core's words may shadow a limb's names.** Today `on` and
`my` became keywords, and enum cases named `on` were renamed. With
third-party cores, listing one could break code that used its word as a
name. (a) A core's words are reserved only in limbs that list it
(recommended: listing is the opt-in, and the message names the core);
(b) a core's words never reserve anything, and are found by position;
(c) as today, a word becomes a keyword everywhere.

**Decided: (a) and (b), the core choosing** (owner: "`a`, `b` depends on
the core"): a core's words are reserved only in limbs that list it, and a
core may instead leave a word unreserved, found by its position.

## 4. Powers: how grants flow and are checked

All engineering, given the rulings, except where marked:

1. **The catalogue.** Signatar's `powers()` gives the tree it defines
   (`inline_assembly`, `c_calls.functions`, `raw_memory.read`, the four
   extension powers, …). Each listed core's `powers()` adds entries under
   the paths it may add to. Signatar decides where a core may add; a core
   cannot add above or beside a Signatar power it is not allowed to
   extend, and nothing lifts Signatar's limits (borrows, Precog).
2. **Grants.** Signatar grants its defaults to every limb (the three
   dangerous powers). Each core grants to every limb built with it, and
   may grant more to favourites named by registry ID. **A core may also
   take a power away** from every limb built with it (owner, 2026-10-07:
   "`a`"): a taken power is not granted, whoever else grants it, and the
   message names the core that took it.
3. **Listing.** Each limb's manifest lists the powers it uses
   (`powers = [cca::Power.c_calls,]`); the program's own limb too.
4. **Using.** The codextra reports every use as `(limb, power path,
   thing, span)`: `c_calls` with thing `"sqrt"`, `vouch` at a `!check`.
5. **The base's check** of each use, in this order, each with its own
   message: granted (by Signatar or a listed core)? listed by the limb?
   not denied by the program (`deny`, with `.except[…]` and `.for[…]`)?
   A lone file skips 1 to 3: it has every core and power.
6. **The record.** The build writes the list of uses per limb, which the
   registry shows and which an upgrade compares ("this version starts
   using `c_calls`").

**Q3. Whether a third-party core's contracts are trusted by Precog.** A
core's operation comes with a contract ("returns at most its first
argument"). If Precog trusts it, every proof downstream depends on that
core's honesty. (a) Trusted, the core being trusted already by the
warning's terms; (b) trusted only for verified cores, an unverified core's
operations being opaque calls whose results Precog knows nothing about
(recommended: proofs then never rest on unverified code, at the price of
more run-time checks where such a core is used); (c) trusted, and every
site resting on one reported in the build's record.

**Decided: (a), every core's contracts trusted, official or not** (owner:
"Holy registry lock in" of trusting only verified ones, and on trust by an
RRx2 proof, "they're just gonna fork our entire project instead"; then
"`a`"). Precog treats a listed core's contracts as true, as it does
IEG's; trusting a core is listing it, as the approved warning says.

## 5. Meaning and IR

**(engineering)** A core's node lowers in one of three ways, chosen per
node:

1. **To Signatar code**: the node is a call to a function in the core's
   CyborgPL part, or plain Signatar it builds. Signatar then types it,
   checks borrows, runs Precog and makes IR as for any code. This is the
   default, and the one every third-party core should reach for.
2. **To IR with a contract**: the core gives an LLVM IR template, and a
   contract Precog uses (Q3). This is IEG's way (`inline.LLVM`, the raw
   operations), and the reason RRx2 checks IEG's IR against its
   contracts.
3. **To runtime**: a call into a static library or bitcode the core ships
   (MCPS's thread pool, Metal's command queues).

What a core may not do: replace Signatar's types, borrow rules or Precog,
or reach into another core's nodes. A core that needs another (MCPS needs
IEG) calls it through its CyborgPL part or its declared nodes.

**The deep official cores.** ASP changes what a literal and plain maths
*are*; Genny adds type variables to the type checker; IEG decides what
needs a mark. None of that is a node to lower.

**Q4. How the deep official cores move out.**

- **(a) Policies Signatar owns, switched on by the core.** ASP's,
  Genny's and IEG's rules stay in Signatar's code; the ASP plugin is the
  switch, plus its syntax hooks. The plugin interface is honest about
  this: Signatar defines a few *policies* (number meaning, generics, the
  marking rule) that only its official cores may enable.
- **(b) Real separation**: Signatar exposes its type checker and Precog
  through the interface, deep enough for ASP and Genny to be written
  outside it. Anyone could then write a different ASP. It is a far larger
  and far less stable interface: every change to the checker is a change
  to the plugin API.
- **(c) Some of each**: (a) now, and a policy moves to (b) the day a
  second author wants to write one.

**Decided: (b), real separation** (owner: "`b`"): Signatar opens its type
checker and Precog far enough that ASP, Genny and IEG are written outside
it, and anyone could write an ASP2. The cost is accepted: a large
interface, where a change to the checker is a change to the plugin API.
(The recommendation had been (c), starting as (a).) Step 6 below is this
separation.

## 6. Finding, loading and versioning plugins

**Q5. What a plugin is, physically.**

- **(a) A native library (`.dylib`, `.so`, `.dll`) behind a C ABI.** Rust
  has no stable ABI between compiler versions, so the boundary is `extern
  "C"` functions and `#[repr(C)]` data (or a crate like `abi_stable`),
  versioned by an interface number. The fastest, and a core can do
  anything the compiler can, including crash it. One build per target
  (macOS ARM, Linux x86, …).
- **(b) A WebAssembly component** (the WebAssembly component model, run by
  `wasmtime`). A core is written in Rust and compiled to `wasm32-wasip2`,
  so "Rust plugin" still holds. One file runs on every machine, the core
  runs in a sandbox (it cannot read files or the network unless the base
  hands it something), and the interface is described in a typed IDL
  (WIT) that versions cleanly. Slower per call, which matters for a parse
  hook called per token, not for one called per construct; and `wasmtime`
  adds weight to CCA.
- **(c) A separate process** spoken to over a pipe (as an editor speaks to
  a language server). Any language, and a crash cannot take the compiler
  down; the slowest, and awkward for fine-grained hooks.

**Decided: (a), native libraries behind a stable C ABI, for every core;
no WebAssembly sandbox** (owner: "the OS lib thingies file type"; asking,
on the way, why the language was getting restricted). A core is trusted
when it is listed, as the approved warning says. The official cores are
still built into CCA, and Signatar into CCA as the default codextra (not
asked; as proposed). **Recommendation, as it was: (b) for every core that is not official; the official
cores built into CCA as Rust crates behind the same trait.** The registry
then ships one file per core version, a stranger's core cannot read your
disk while it compiles your code, and the official cores pay nothing. The
warning stays exactly as approved: a sandboxed core can still change what
your program does.

**(engineering)**, given any of the three:

- **Where they live.** Installed cores in the machine-wide folder beside
  `installed.cplr`: `…/CyborgPL/cores/<id>/<version>/`, holding the plugin
  file, its CyborgPL part, and a `Core.cplr` manifest (name, ID, version,
  the interface version it was built for, needs, powers it adds). A
  `cca::Core[path = …]` points at such a folder. Synthia downloads into
  it; CCA never downloads (as for std).
- **Two versions.** The core's own version (which release, the date and
  time) and the interface version (which (B) it speaks). CCA loads a core
  whose interface version it supports, and otherwise names both.
- **Pinning.** A lockfile beside `CCA.cplr` records each core's content
  hash, so the same manifest always loads the same bytes.
- **Verified cores** are the official ones, built into CCA and checked as
  CCA is (RRx2 for IEG's IR). Any other core gets the approved warning.

**Q6. Whether an unverified core may run at all in a build with
`--assert-proofs`, or in the registry's own build of a published limb.**
(a) Yes, with the warning, as everywhere; (b) refused there, those builds
being where trust is checked (recommended: a published limb's build on
the registry then never runs strangers' code).

**Decided: (a), an unofficial core runs there too**, and the registry
builds inside a throwaway, sealed machine, so a native core cannot harm
its servers (owner: "yea", to design claude's suggestion).

## 7. What changes in CCA, and in what order

Each step leaves CCA working and the suite green (engineering, unless a
question above changes it):

1. **Split the crate**: `cca-base` (files, manifests, loading, the power
   check, the LLVM handoff), `signatar` (the rest of today's compiler),
   and `cores/*` (one crate per official core, empty at first). No
   behaviour changes.
2. **The power ledger**, before any plugin: Signatar reports uses, the
   base checks them against `powers` and `deny`. This is the powers
   ruling built, and needs nothing below.
3. **The hook menu** (Q1) inside Signatar, with today's switches rewired
   through it: the gate tables become claims made by `cores/asp`,
   `cores/mcps`, and so on.
4. **Move WSTN out** first (only name characters), then the slimmer
   base's word-only cores (TAS, AI, WSMW, Money, UniTect, CAPS): the
   smallest cores, proving the menu. (MTM was to be first; it is gone.)
5. **MCPS's syntax and runtime**, then the **GPU cores**, which are mostly
   new syntax lowering to runtime calls (section 5, way 3).
6. **ASP, IEG, Genny** as Q4 decides.
7. **External loading** (Q5: native libraries, C ABI) last, once the official cores have used the
   interface long enough to stop it moving; then the warning, the
   lockfile, and `cca::Core[path = …]` working.

## 8. Gaps that block parts of this (the owner's)

1. **Which powers MCPS, AutoGPU / Metal / Nvidia / Vulkan, WSTN and the
   new cores add, and which a core may take away.** Section 4 can be built without them, but each core's
   `powers()` cannot be written, and step 5 cannot start, until it is
   known whether (say) spawning threads, a GPU queue, or a name in another
   script is a power a program may deny.
2. **IEG's raw operations** (`add`, `negate`, `compare` are design
   claude's sketch). Their names, which types they take, and their
   contracts decide IEG's nodes and the IR templates RRx2 checks; IEG
   cannot move out (step 6) before them.

(The question of loading another codextra is gone with codextras,
2026-10-07: a deeper change than a core can make is a fork.)
