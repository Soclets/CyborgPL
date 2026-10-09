# Plugins: how lobes plug into CCA (proposal)

**A core is a lobe since 2026-10-09** (owner: "Lobe next", then "yea, go"): a Neuron lists `lobes = [kin::Lobe[…]]`. The crates under `cores/` keep their folder names for now.

**Since 2026-10-07 (owner; DECISIONS 2026-10-07, "No base and no
codextras")**: there is no base below the language and no codextras.
Brainstem is the lowest level and does what this proposal gave the base
(reading files and manifests, loading lobes, checking powers, handing IR
to LLVM); dialects were tried and dropped the same day ("just let them
fork so they could do whatever they want"). A lobe may also **take powers
away** (owner: "`a`"; a game engine's lobe forbidding raw memory to the
limbs built with it). So the three boundaries below are one, Brainstem ↔
lobe; (A) and (C) are inside Brainstem now, and the `cca-base` crate from
step 1 is Brainstem's power ledger, not a layer under it. The rename of
Brainstem to "base CyborgPL" is recorded and not applied ("Record, don't
implement yet"). MTM is gone (its spellings are old ones, refused); the
slimmer base added TAS, AI, WSMW, Money, UniTect and CAPS, and ASP took
the range types and length bounds (section 1). The text below is the
proposal as answered, with "the base" and "the codextra" read as
Brainstem.

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
2026-10-07): the base reads files, loads the codextra and lobes, enforces
grants, and hands LLVM IR to LLVM unread; the codextra (Brainstem) does
the syntax, types, borrows, Cortex and the IR; the codextra limits what
lobes may do; lobes grant powers, add syntax and depend on lobes; every
limb lists the powers it uses; a program may only `deny`; a lone file has
every lobe and power; lobes are named by registry ID and version.

## 1. Where CCA is today

Each official lobe is a switch inside one compiler. Listing ASP lets the
parser accept `to.` and makes literals true values in the checker;
listing MTM lets the lexer's alternative spellings through; listing MCPS
lets `concur` and `for para` parse; and so on. Concretely, today:

| Lobe | Where it lives in CCA now |
|---|---|
| ASP | `to.` and `check` in the parser's word gate; literal meaning in `check/expr.rs` (`raw_files`); `to.T.bits`; range types and length bounds (the type parser) |
| IEG | `no_check` / `NO_FUCKING_CHECKS` in the gate; the marking rule (`needs_mark`); `on.machine` |
| Genny | generics and `interface` in the gate, then all of `check/` |
| MCPS | `concur`, `whichever`, `channel`, `for para` in the gate; `par.rs`, `codegen/par.rs`, `codegen/tasks.rs` |
| WSTN | a parser pre-scan refusing a name past ASCII |
| TAS, AI, WSMW, Money, UniTect, CAPS | words in the gate (`test fn`, `must`, `brain16`, `int128`, `not_std.`, `deci64`, `char.grapheme`, `case.insensitive`, …); their meaning still all of `check/` |
| cord | not a lobe: a limb CCA does not have yet (`cord::` is a stub) |
| MTM | gone (2026-10-07): its spellings are old ones, refused by name in every limb |

So "moving the lobes out" is two different jobs: the thin ones (WSTN,
the gate words, the syntax half of MCPS) are already almost separate; the deep ones
(ASP, IEG, Genny) are woven through the type checker, Cortex and the IR,
because they change what a number or a type *means*. The proposal treats
them differently on purpose.

## 2. The three interfaces

There are three boundaries, not one:

```
base  <──(A)──>  codextra (Brainstem)  <──(B)──>  lobes
  └────────────────────(C)─────────────────────┘
```

- **(A) base ↔ codextra**: small and stable. The base gives the codextra
  the files, the manifests and the loaded lobes; the codextra gives back
  LLVM IR, a list of power uses, and messages.
- **(B) codextra ↔ lobe**: the big one, and **the codextra's to define**,
  since "a codextra defines which powers exist and limits lobes". A lobe
  written for Brainstem is a Brainstem lobe; another codextra defines its
  own (B). So (B) below is *Brainstem's* lobe interface.
- **(C) base ↔ lobe**: only loading, identity, dependencies and the power
  catalogue, never syntax or IR. The base enforces grants, so it must
  know which powers each lobe adds and grants, without understanding
  them.

### (A) What a codextra provides (engineering, in outline)

```rust
pub trait Codextra {
    fn id(&self) -> RegistryId;                // and version
    fn read_manifest(&self, text: &str) -> Result<Manifest, Message>;
    fn powers(&self) -> PowerCatalogue;         // the powers that exist
    fn defaults(&self) -> Vec<PowerGrant>;      // e.g. c_calls to every limb
    fn build(&self, job: &Job, lobes: &[&dyn Lobe]) -> Result<Built, Vec<Message>>;
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
(cores.md: Brainstem reads `Kinesin.nrp`).

### (B) What a Brainstem lobe provides

A lobe is two parts, as cores.md has it: **Rust** (what the compiler runs)
and **CyborgPL** (code it brings, compiled as a limb of its own). The Rust
part answers four kinds of question:

1. **Which syntax it adds**: entries in Brainstem's hook menu (section 3).
2. **What its syntax means**: each hook's node lowers to Brainstem code,
   to calls into the lobe's own CyborgPL part, or to IR (section 5).
3. **Which powers it adds and grants**: its catalogue entries, and its
   grants (to every limb, or to favourites by registry ID).
4. **Contracts** for Cortex: what each operation it adds promises and
   requires, so Cortex can reason through it (section 5).

```rust
pub trait Lobe {
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

**Q1. How a lobe adds syntax to Brainstem's parser.**

- **(a) A fixed menu of hooks.** Brainstem names the places a lobe may
  extend, and a lobe claims entries. The menu, read off what the official
  lobes already need:

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

  Brainstem keeps precedence, spans and error messages; two lobes claiming
  the same entry are refused when the manifest is read, naming both.
  New *kinds* of syntax need a new entry in Brainstem.

- **(b) Token rewriting**, like Rust's procedural macros: a lobe is given
  the tokens after its word, up to a matching close, and gives back plain
  Brainstem tokens (calling into its CyborgPL part). Any shape fits, but
  messages point at rewritten code, Cortex sees only the expansion, and
  two lobes' rewrites can interfere in ways neither checks.

- **(c) Grammar fragments**: each lobe ships grammar rules that Brainstem
  merges into its parser. The most expressive, but Brainstem's parser is
  hand-written recursive descent and would be rewritten as a generated
  one; ambiguity between two lobes' grammars is found only by the
  generator, if at all; and errors get worse.

**Decided: (a), with (b) inside the statement-word and prefix-head hooks
only** (owner: "yea"); two lobes claiming one entry are refused when the
manifest is read. **Recommendation, as it was: (a), with (b) inside the statement-word and prefix-head
hooks only.** Every syntax the official lobes have fits (a); a hook's
parse may take a balanced run of tokens raw, which covers the odd shape
without opening the whole grammar. Conflicts are caught at load, where a
user can act on them.

**Q2. Whether a lobe's words may shadow a limb's names.** Today `on` and
`my` became keywords, and enum cases named `on` were renamed. With
third-party lobes, listing one could break code that used its word as a
name. (a) A lobe's words are reserved only in limbs that list it
(recommended: listing is the opt-in, and the message names the lobe);
(b) a lobe's words never reserve anything, and are found by position;
(c) as today, a word becomes a keyword everywhere.

**Decided: (a) and (b), the lobe choosing** (owner: "`a`, `b` depends on
the lobe"): a lobe's words are reserved only in limbs that list it, and a
lobe may instead leave a word unreserved, found by its position.

## 4. Powers: how grants flow and are checked

All engineering, given the rulings, except where marked:

1. **The catalogue.** Brainstem's `powers()` gives the tree it defines
   (`inline_assembly`, `c_calls.functions`, `raw_memory.read`, the four
   extension powers, …). Each listed lobe's `powers()` adds entries under
   the paths it may add to. Brainstem decides where a lobe may add; a lobe
   cannot add above or beside a Brainstem power it is not allowed to
   extend, and nothing lifts Brainstem's limits (borrows, Cortex).
2. **Grants.** Brainstem grants its defaults to every limb (the three
   dangerous powers). Each lobe grants to every limb built with it, and
   may grant more to favourites named by registry ID. **A lobe may also
   take a power away** from every limb built with it (owner, 2026-10-07:
   "`a`"): a taken power is not granted, whoever else grants it, and the
   message names the lobe that took it.
3. **Listing.** Each limb's manifest lists the powers it uses
   (`powers = [kin::Power.c_calls,]`); the program's own limb too.
4. **Using.** The codextra reports every use as `(limb, power path,
   thing, span)`: `c_calls` with thing `"sqrt"`, `vouch` at a `!check`.
5. **The base's check** of each use, in this order, each with its own
   message: granted (by Brainstem or a listed lobe)? listed by the limb?
   not denied by the program (`deny`, with `.except[…]` and `.for[…]`)?
   A lone file skips 1 to 3: it has every lobe and power.
6. **The record.** The build writes the list of uses per limb, which the
   registry shows and which an upgrade compares ("this version starts
   using `c_calls`").

**Q3. Whether a third-party lobe's contracts are trusted by Cortex.** A
lobe's operation comes with a contract ("returns at most its first
argument"). If Cortex trusts it, every proof downstream depends on that
lobe's honesty. (a) Trusted, the lobe being trusted already by the
warning's terms; (b) trusted only for verified lobes, an unverified lobe's
operations being opaque calls whose results Cortex knows nothing about
(recommended: proofs then never rest on unverified code, at the price of
more run-time checks where such a lobe is used); (c) trusted, and every
site resting on one reported in the build's record.

**Decided: (a), every lobe's contracts trusted, official or not** (owner:
"Holy registry lock in" of trusting only verified ones, and on trust by an
RRx2 proof, "they're just gonna fork our entire project instead"; then
"`a`"). Cortex treats a listed lobe's contracts as true, as it does
IEG's; trusting a lobe is listing it, as the approved warning says.

## 5. Meaning and IR

**(engineering)** A lobe's node lowers in one of three ways, chosen per
node:

1. **To Brainstem code**: the node is a call to a function in the lobe's
   CyborgPL part, or plain Brainstem it builds. Brainstem then types it,
   checks borrows, runs Cortex and makes IR as for any code. This is the
   default, and the one every third-party lobe should reach for.
2. **To IR with a contract**: the lobe gives an LLVM IR template, and a
   contract Cortex uses (Q3). This is IEG's way (`inline.LLVM`, the raw
   operations), and the reason RRx2 checks IEG's IR against its
   contracts.
3. **To runtime**: a call into a static library or bitcode the lobe ships
   (MCPS's thread pool, Metal's command queues).

What a lobe may not do: replace Brainstem's types, borrow rules or Cortex,
or reach into another lobe's nodes. A lobe that needs another (MCPS needs
IEG) calls it through its CyborgPL part or its declared nodes.

**The deep official lobes.** ASP changes what a literal and plain maths
*are*; Genny adds type variables to the type checker; IEG decides what
needs a mark. None of that is a node to lower.

**Q4. How the deep official lobes move out.**

- **(a) Policies Brainstem owns, switched on by the lobe.** ASP's,
  Genny's and IEG's rules stay in Brainstem's code; the ASP plugin is the
  switch, plus its syntax hooks. The plugin interface is honest about
  this: Brainstem defines a few *policies* (number meaning, generics, the
  marking rule) that only its official lobes may enable.
- **(b) Real separation**: Brainstem exposes its type checker and Cortex
  through the interface, deep enough for ASP and Genny to be written
  outside it. Anyone could then write a different ASP. It is a far larger
  and far less stable interface: every change to the checker is a change
  to the plugin API.
- **(c) Some of each**: (a) now, and a policy moves to (b) the day a
  second author wants to write one.

**Decided: (b), real separation** (owner: "`b`"): Brainstem opens its type
checker and Cortex far enough that ASP, Genny and IEG are written outside
it, and anyone could write an ASP2. The cost is accepted: a large
interface, where a change to the checker is a change to the plugin API.
(The recommendation had been (c), starting as (a).) Step 6 below is this
separation.

## 6. Finding, loading and versioning plugins

**Q5. What a plugin is, physically.**

- **(a) A native library (`.dylib`, `.so`, `.dll`) behind a C ABI.** Rust
  has no stable ABI between compiler versions, so the boundary is `extern
  "C"` functions and `#[repr(C)]` data (or a crate like `abi_stable`),
  versioned by an interface number. The fastest, and a lobe can do
  anything the compiler can, including crash it. One build per target
  (macOS ARM, Linux x86, …).
- **(b) A WebAssembly component** (the WebAssembly component model, run by
  `wasmtime`). A lobe is written in Rust and compiled to `wasm32-wasip2`,
  so "Rust plugin" still holds. One file runs on every machine, the lobe
  runs in a sandbox (it cannot read files or the network unless the base
  hands it something), and the interface is described in a typed IDL
  (WIT) that versions cleanly. Slower per call, which matters for a parse
  hook called per token, not for one called per construct; and `wasmtime`
  adds weight to CCA.
- **(c) A separate process** spoken to over a pipe (as an editor speaks to
  a language server). Any language, and a crash cannot take the compiler
  down; the slowest, and awkward for fine-grained hooks.

**Decided: (a), native libraries behind a stable C ABI, for every lobe;
no WebAssembly sandbox** (owner: "the OS lib thingies file type"; asking,
on the way, why the language was getting restricted). A lobe is trusted
when it is listed, as the approved warning says. The official lobes are
still built into CCA, and Brainstem into CCA as the default codextra (not
asked; as proposed). **Recommendation, as it was: (b) for every lobe that is not official; the official
lobes built into CCA as Rust crates behind the same trait.** The registry
then ships one file per lobe version, a stranger's lobe cannot read your
disk while it compiles your code, and the official lobes pay nothing. The
warning stays exactly as approved: a sandboxed lobe can still change what
your program does.

**(engineering)**, given any of the three:

- **Where they live.** Installed lobes in the machine-wide folder beside
  `installed.cplr`: `…/CyborgPL/lobes/<id>/<version>/`, holding the plugin
  file, its CyborgPL part, and a `Lobe.cplr` manifest (name, ID, version,
  the interface version it was built for, needs, powers it adds). A
  `kin::Lobe[path = …]` points at such a folder. Synthia downloads into
  it; CCA never downloads (as for std).
- **Two versions.** The lobe's own version (which release, the date and
  time) and the interface version (which (B) it speaks). CCA loads a lobe
  whose interface version it supports, and otherwise names both.
- **Pinning.** A lockfile beside `Kinesin.nrp` records each lobe's content
  hash, so the same manifest always loads the same bytes.
- **Verified lobes** are the official ones, built into CCA and checked as
  CCA is (RRx2 for IEG's IR). Any other lobe gets the approved warning.

**Q6. Whether an unverified lobe may run at all in a build with
`--assert-proofs`, or in the registry's own build of a published limb.**
(a) Yes, with the warning, as everywhere; (b) refused there, those builds
being where trust is checked (recommended: a published limb's build on
the registry then never runs strangers' code).

**Decided: (a), an unofficial lobe runs there too**, and the registry
builds inside a throwaway, sealed machine, so a native lobe cannot harm
its servers (owner: "yea", to design claude's suggestion).

## 7. What changes in CCA, and in what order

Each step leaves CCA working and the suite green (engineering, unless a
question above changes it):

1. **Split the crate**: `cca-base` (files, manifests, loading, the power
   check, the LLVM handoff), `signatar` (the rest of today's compiler),
   and `cores/*` (one crate per official lobe, empty at first). No
   behaviour changes.
2. **The power ledger**, before any plugin: Brainstem reports uses, the
   base checks them against `powers` and `deny`. This is the powers
   ruling built, and needs nothing below.
3. **The hook menu** (Q1) inside Brainstem, with today's switches rewired
   through it: the gate tables become claims made by `cores/asp`,
   `cores/mcps`, and so on.
4. **Move WSTN out** first (only name characters), then the slimmer
   base's word-only lobes (TAS, AI, WSMW, Money, UniTect, CAPS): the
   smallest lobes, proving the menu. (MTM was to be first; it is gone.)
5. **MCPS's syntax and runtime**, then the **GPU lobes**, which are mostly
   new syntax lowering to runtime calls (section 5, way 3).
6. **ASP, IEG, Genny** as Q4 decides.
7. **External loading** (Q5: native libraries, C ABI) last, once the official lobes have used the
   interface long enough to stop it moving; then the warning, the
   lockfile, and `kin::Lobe[path = …]` working.

## 8. Gaps that block parts of this (the owner's)

1. **Which powers MCPS, AutoGPU / Metal / Nvidia / Vulkan, WSTN and the
   new lobes add, and which a lobe may take away.** Section 4 can be built without them, but each lobe's
   `powers()` cannot be written, and step 5 cannot start, until it is
   known whether (say) spawning threads, a GPU queue, or a name in another
   script is a power a program may deny.
2. **IEG's raw operations** (`add`, `negate`, `compare` are design
   claude's sketch). Their names, which types they take, and their
   contracts decide IEG's nodes and the IR templates RRx2 checks; IEG
   cannot move out (step 6) before them.

(The question of loading another codextra is gone with codextras,
2026-10-07: a deeper change than a lobe can make is a fork.)
