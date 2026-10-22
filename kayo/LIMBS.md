# Limbs: how limbs plug into KAYO\n (proposal)

**A core is a limb since 2026-10-09** (owner: "Limb next", then "yea, go"): a kit lists `limbs = [cyjet::Limb[…]]`. The crates are under `limbs/` since the KAYO\n rename (2026-10-09).

**Since 2026-10-07 (owner; DECISIONS 2026-10-07, "No base and no
codextras")**: there is no base below the language and no codextras.
The base is the lowest level and does what this proposal gave the base
(reading files and manifests, loading limbs, checking powers, handing IR
to LLVM); dialects were tried and dropped the same day ("just let them
fork so they could do whatever they want"). A limb may also **take powers
away** (owner: "`a`"; a game engine's limb forbidding raw memory to the
kits built with it). So the three boundaries below are one, the base ↔
limb; (A) and (C) are inside the base now, and the `kayo-base` crate from
step 1 is the base's power ledger, not a layer under it. Ground, as it was named until 2026-10-16, is the base since then. MTM is gone (its spellings are old ones, refused); the
slimmer base added TAS, AI, WSMW, Money, UniTect and CAPS, and ASP took
the range types and length bounds (section 1). **ASP and Genny are
retired** (owner, 2026-10-08; built 2026-10-09): what they held is the base's, so a manifest listing either is refused.
**Limbs go by expertise since 2026-10-12** (owner): IEG and TAS are the base's too, and WSTN, UniTect and CAPS are one limb, unicode; CNC holds
everything about C (2026-10-10). The text below is the
proposal as answered, with "the base" and "the codextra" read as
the base.

Compiler claude's proposal, 2026-10-07, for the owner to approve
(2026-10-06: "compiler claude proposes the limb interface, with the
owner approving"). **The owner answered all six questions the same day**
(DECISIONS 2026-10-07, "The limb interface"); each answer is under its
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
2026-10-07): the base reads files, loads the codextra and limbs, enforces
grants, and hands LLVM IR to LLVM unread; the codextra (the base) does
the syntax, types, borrows, the prover and the IR; the codextra limits what
limbs may do; limbs grant powers, add syntax and depend on limbs; every
kit lists the powers it uses; a program may only `deny`; a lone file has
every limb and power; limbs are named by registry ID and version.

## 1. Where KAYO\n is today

Each official limb is a switch inside one compiler. Listing MTM lets the lexer's alternative spellings through; listing MCPS
lets `concur` and `for para` parse; and so on. Concretely, today:

| Limb | Where it lives in KAYO\n now |
|---|---|
| MCPS | `concur`, `whichever`, `channel`, `for para` in the gate; `par.rs`, `codegen/par.rs`, `codegen/tasks.rs` |
| unicode | a parser pre-scan refusing a name past ASCII (was WSTN); `str.utf16`, `char.grapheme`, counting and `case.any` in the gate (was UniTect and CAPS) |
| CNC | `extern.C`, `incl.C`, `layout.C` and the `C.` types in the gate; `cheader.rs`, the `C.` types in `target.rs` |
| AI, WSMW, Money | words in the gate (`brain16`, `i128`, `not_std.`, `d64`, …); their meaning still all of `check/` |
| IEG, TAS | retired (2026-10-12): `no_check`, the marking rule in every kit, `on.machine`, `test fn` and `must` are the base's, which grants `vouch` and the operator powers to every kit |
| std | not a limb: a kit KAYO\n does not have yet (`std::` is a stub) |
| MTM | gone (2026-10-07): its spellings are old ones, refused by name in every kit |
| ASP, Genny | retired (2026-10-08): `to.`, `check`, true-value literals, range types, generics and interfaces are the base's; their IDs are refused, naming why |

So "moving the limbs out" is two different jobs: the thin ones (WSTN,
the gate words, the syntax half of MCPS) are already almost separate; the deep ones
(IEG, and ASP and Genny until they were retired) are woven through the type checker, the prover and the IR,
because they change what a number or a type *means*. The proposal treats
them differently on purpose.

## 2. The three interfaces

There are three boundaries, not one:

```
base  <──(A)──>  codextra (the base)  <──(B)──>  limbs
  └────────────────────(C)─────────────────────┘
```

- **(A) base ↔ codextra**: small and stable. The base gives the codextra
  the files, the manifests and the loaded limbs; the codextra gives back
  LLVM IR, a list of power uses, and messages.
- **(B) codextra ↔ limb**: the big one, and **the codextra's to define**,
  since "a codextra defines which powers exist and limits limbs". A limb
  written for the base is a the base limb; another codextra defines its
  own (B). So (B) below is *the base's* limb interface.
- **(C) base ↔ limb**: only loading, identity, dependencies and the power
  catalogue, never syntax or IR. The base enforces grants, so it must
  know which powers each limb adds and grants, without understanding
  them.

### (A) What a codextra provides (engineering, in outline)

```rust
pub trait Codextra {
    fn id(&self) -> RegistryId;                // and version
    fn read_manifest(&self, text: &str) -> Result<Manifest, Message>;
    fn powers(&self) -> PowerCatalogue;         // the powers that exist
    fn defaults(&self) -> Vec<PowerGrant>;      // e.g. c_calls to every kit
    fn build(&self, job: &Job, limbs: &[&dyn Limb]) -> Result<Built, Vec<Message>>;
}

pub struct Built {
    pub ir: LlvmModule,          // textual IR or bitcode, handed on unread
    pub uses: Vec<PowerUse>,     // (kit, power path, thing, where)
    pub messages: Vec<Message>,  // warnings and notes
}
```

The base checks `uses` against the manifests (section 4) before handing
the IR to LLVM, so a codextra cannot use a power nobody granted by
forgetting to check: the base does not trust it to. `read_manifest`
is the codextra's because the base does not know what a quoted name is
(cores.md: the base reads `CyJet.toml`).

### (B) What a the base limb provides

A limb is two parts, as cores.md has it: **Rust** (what the compiler runs)
and **CyborgPL** (code it brings, compiled as a kit of its own). The Rust
part answers four kinds of question:

1. **Which syntax it adds**: entries in the base's hook menu (section 3).
2. **What its syntax means**: each hook's node lowers to the base code,
   to calls into the limb's own CyborgPL part, or to IR (section 5).
3. **Which powers it adds and grants**: its catalogue entries, and its
   grants (to every kit, or to favourites by registry ID).
4. **Contracts** for the prover: what each operation it adds promises and
   requires, so the prover can reason through it (section 5).

```rust
pub trait Limb {
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

**Q1. How a limb adds syntax to the base's parser.**

- **(a) A fixed menu of hooks.** the base names the places a limb may
  extend, and a limb claims entries. The menu, read off what the official
  limbs already need:

  | Hook | Example today |
  |---|---|
  | a statement word | `concur …`, `whichever …`, `check …` |
  | a prefix head, `head.…[…]` | `to.i8[…]`, `to.i8.bits[…]` |
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

  the base keeps precedence, spans and error messages; two limbs claiming
  the same entry are refused when the manifest is read, naming both.
  New *kinds* of syntax need a new entry in the base.

- **(b) Token rewriting**, like Rust's procedural macros: a limb is given
  the tokens after its word, up to a matching close, and gives back plain
  the base tokens (calling into its CyborgPL part). Any shape fits, but
  messages point at rewritten code, the prover sees only the expansion, and
  two limbs' rewrites can interfere in ways neither checks.

- **(c) Grammar fragments**: each limb ships grammar rules that the base
  merges into its parser. The most expressive, but the base's parser is
  hand-written recursive descent and would be rewritten as a generated
  one; ambiguity between two limbs' grammars is found only by the
  generator, if at all; and errors get worse.

**Decided: (a), with (b) inside the statement-word and prefix-head hooks
only** (owner: "yea"); two limbs claiming one entry are refused when the
manifest is read. **Recommendation, as it was: (a), with (b) inside the statement-word and prefix-head
hooks only.** Every syntax the official limbs have fits (a); a hook's
parse may take a balanced run of tokens raw, which covers the odd shape
without opening the whole grammar. Conflicts are caught at load, where a
user can act on them.

**Q2. Whether a limb's words may shadow a kit's names.** Today `on` and
`my` became keywords, and enum cases named `on` were renamed. With
third-party limbs, listing one could break code that used its word as a
name. (a) A limb's words are reserved only in kits that list it
(recommended: listing is the opt-in, and the message names the limb);
(b) a limb's words never reserve anything, and are found by position;
(c) as today, a word becomes a keyword everywhere.

**Decided: (a) and (b), the limb choosing** (owner: "`a`, `b` depends on
the limb"): a limb's words are reserved only in kits that list it, and a
limb may instead leave a word unreserved, found by its position.

## 4. Powers: how grants flow and are checked

All engineering, given the rulings, except where marked:

1. **The catalogue.** the base's `powers()` gives the tree it defines
   (`inline_assembly`, `c_calls.functions`, `raw_memory.read`, the four
   extension powers, …). Each listed limb's `powers()` adds entries under
   the paths it may add to. The base decides where a limb may add; a limb
   cannot add above or beside a the base power it is not allowed to
   extend, and nothing lifts the base's limits (borrows, the prover).
2. **Grants.** the base grants its defaults to every kit (the three
   dangerous powers). Each limb grants to every kit built with it, and
   may grant more to favourites named by registry ID. **A limb may also
   take a power away** from every kit built with it (owner, 2026-10-07:
   "`a`"): a taken power is not granted, whoever else grants it, and the
   message names the limb that took it.
3. **Listing.** Each kit's manifest lists the powers it uses
   (`powers = [cyjet::Power.c_calls,]`); the program's own kit too.
4. **Using.** The codextra reports every use as `(kit, power path,
   thing, span)`: `c_calls` with thing `"sqrt"`, `vouch` at a `!check`.
5. **The base's check** of each use, in this order, each with its own
   message: granted (by the base or a listed limb)? listed by the kit?
   not denied by the program (`deny`, with `.except[…]` and `.for[…]`)?
   A lone file skips 1 to 3: it has every limb and power.
6. **The record.** The build writes the list of uses per kit, which the
   registry shows and which an upgrade compares ("this version starts
   using `c_calls`").

**Q3. Whether a third-party limb's contracts are trusted by the prover.** A
limb's operation comes with a contract ("returns at most its first
argument"). If the prover trusts it, every proof downstream depends on that
limb's honesty. (a) Trusted, the limb being trusted already by the
warning's terms; (b) trusted only for verified limbs, an unverified limb's
operations being opaque calls whose results the prover knows nothing about
(recommended: proofs then never rest on unverified code, at the price of
more run-time checks where such a limb is used); (c) trusted, and every
site resting on one reported in the build's record.

**Decided: (a), every limb's contracts trusted, official or not** (owner:
"Holy registry lock in" of trusting only verified ones, and on trust by an
RRx2 proof, "they're just gonna fork our entire project instead"; then
"`a`"). the prover treats a listed limb's contracts as true, as it does
IEG's; trusting a limb is listing it, as the approved warning says.

## 5. Meaning and IR

**(engineering)** A limb's node lowers in one of three ways, chosen per
node:

1. **To the base code**: the node is a call to a function in the limb's
   CyborgPL part, or plain the base it builds. The base then types it,
   checks borrows, runs the prover and makes IR as for any code. This is the
   default, and the one every third-party limb should reach for.
2. **To IR with a contract**: the limb gives an LLVM IR template, and a
   contract the prover uses (Q3). This is IEG's way (`inline.LLVM`, the raw
   operations), and the reason RRx2 checks IEG's IR against its
   contracts.
3. **To runtime**: a call into a static library or bitcode the limb ships
   (MCPS's thread pool, Metal's command queues).

What a limb may not do: replace the base's types, borrow rules or the prover,
or reach into another limb's nodes. A limb that needs another (MCPS needs
IEG) calls it through its CyborgPL part or its declared nodes.

**The deep official limbs.** ASP changes what a literal and plain maths
*are*; Genny adds type variables to the type checker; IEG decides what
needs a mark. None of that is a node to lower.

**Q4. How the deep official limbs move out.**

- **(a) Policies the base owns, switched on by the limb.** ASP's,
  Genny's and IEG's rules stay in the base's code; the ASP limb is the
  switch, plus its syntax hooks. The limb interface is honest about
  this: the base defines a few *policies* (number meaning, generics, the
  marking rule) that only its official limbs may enable.
- **(b) Real separation**: the base exposes its type checker and the prover
  through the interface, deep enough for ASP and Genny to be written
  outside it. Anyone could then write a different ASP. It is a far larger
  and far less stable interface: every change to the checker is a change
  to the limb API.
- **(c) Some of each**: (a) now, and a policy moves to (b) the day a
  second author wants to write one.

**Decided: (b), real separation** (owner: "`b`"): the base opens its type
checker and the prover far enough that ASP, Genny and IEG are written outside
it, and anyone could write an ASP2. The cost is accepted: a large
interface, where a change to the checker is a change to the limb API.
(The recommendation had been (c), starting as (a).) Step 6 below is this
separation.

## 6. Finding, loading and versioning limbs

**Q5. What a limb is, physically.**

- **(a) A native library (`.dylib`, `.so`, `.dll`) behind a C ABI.** Rust
  has no stable ABI between compiler versions, so the boundary is `extern
  "C"` functions and `#[repr(C)]` data (or a crate like `abi_stable`),
  versioned by an interface number. The fastest, and a limb can do
  anything the compiler can, including crash it. One build per target
  (macOS ARM, Linux x86, …).
- **(b) A WebAssembly component** (the WebAssembly component model, run by
  `wasmtime`). A limb is written in Rust and compiled to `wasm32-wasip2`,
  so "Rust limb" still holds. One file runs on every machine, the limb
  runs in a sandbox (it cannot read files or the network unless the base
  hands it something), and the interface is described in a typed IDL
  (WIT) that versions cleanly. Slower per call, which matters for a parse
  hook called per token, not for one called per construct; and `wasmtime`
  adds weight to CCA.
- **(c) A separate process** spoken to over a pipe (as an editor speaks to
  a language server). Any language, and a crash cannot take the compiler
  down; the slowest, and awkward for fine-grained hooks.

**Decided: (a), native libraries behind a stable C ABI, for every limb;
no WebAssembly sandbox** (owner: "the OS lib thingies file type"; asking,
on the way, why the language was getting restricted). A limb is trusted
when it is listed, as the approved warning says. The official limbs are
still built into KAYO\n, and the base into KAYO\n as the default codextra (not
asked; as proposed). **Recommendation, as it was: (b) for every limb that is not official; the official
limbs built into KAYO\n as Rust crates behind the same trait.** The registry
then ships one file per limb version, a stranger's limb cannot read your
disk while it compiles your code, and the official limbs pay nothing. The
warning stays exactly as approved: a sandboxed limb can still change what
your program does.

**(engineering)**, given any of the three:

- **Where they live.** Installed limbs in the machine-wide folder beside
  `installed.cplr`: `…/CyborgPL/limbs/<id>/<version>/`, holding the limb
  file, its CyborgPL part, and a `Limb.cplr` manifest (name, ID, version,
  the interface version it was built for, needs, powers it adds). A
  `cyjet::Limb[path = …]` points at such a folder. Synthia downloads into
  it; KAYO\n never downloads (as for std).
- **Two versions.** The limb's own version (which release, the date and
  time) and the interface version (which (B) it speaks). KAYO\n loads a limb
  whose interface version it supports, and otherwise names both.
- **Pinning.** A lockfile beside `CyJet.toml` records each limb's content
  hash, so the same manifest always loads the same bytes.
- **Verified limbs** are the official ones, built into KAYO\n and checked as
  KAYO\n is (RRx2 for IEG's IR). Any other limb gets the approved warning.

**Q6. Whether an unverified limb may run at all in a build with
`--assert-proofs`, or in the registry's own build of a published kit.**
(a) Yes, with the warning, as everywhere; (b) refused there, those builds
being where trust is checked (recommended: a published kit's build on
the registry then never runs strangers' code).

**Decided: (a), an unofficial limb runs there too**, and the registry
builds inside a throwaway, sealed machine, so a native limb cannot harm
its servers (owner: "yea", to design claude's suggestion).

## 7. What changes in KAYO\n, and in what order

Each step leaves KAYO\n working and the suite green (engineering, unless a
question above changes it):

1. **Split the crate**: `kayo-base` (files, manifests, loading, the power
   check, the LLVM handoff), `signatar` (the rest of today's compiler),
   and `limbs/*` (one crate per official limb, empty at first). No
   behaviour changes.
2. **The power ledger**, before any limb: the base reports uses, the
   base checks them against `powers` and `deny`. This is the powers
   ruling built, and needs nothing below.
3. **The hook menu** (Q1) inside the base, with today's switches rewired
   through it: the gate tables become claims made by `limbs/asp`,
   `limbs/mcps`, and so on.
4. **Move WSTN out** first (only name characters), then the slimmer
   base's word-only limbs (TAS, AI, WSMW, Money, UniTect, CAPS): the
   smallest limbs, proving the menu. (MTM was to be first; it is gone.)
5. **MCPS's syntax and runtime**, then the **GPU limbs**, which are mostly
   new syntax lowering to runtime calls (section 5, way 3).
6. **IEG** as Q4 decides (ASP and Genny are retired: the base's).
7. **External loading** (Q5: native libraries, C ABI) last, once the official limbs have used the
   interface long enough to stop it moving; then the warning, the
   lockfile, and `cyjet::Limb[path = …]` working.

## 8. Gaps that block parts of this (the owner's)

1. **Which powers MCPS, AutoGPU / Metal / Nvidia / Vulkan, WSTN and the
   new limbs add, and which a limb may take away.** Section 4 can be built without them, but each limb's
   `powers()` cannot be written, and step 5 cannot start, until it is
   known whether (say) spawning threads, a GPU queue, or a name in another
   script is a power a program may deny.
2. **IEG's raw operations** (`add`, `negate`, `compare` are design
   claude's sketch). Their names, which types they take, and their
   contracts decide IEG's nodes and the IR templates RRx2 checks; IEG
   cannot move out (step 6) before them.

(The question of loading another codextra is gone with codextras,
2026-10-07: a deeper change than a limb can make is a fork.)
