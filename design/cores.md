# Signatar, cores and limbs (owner, 2026-10-05/06/07; being designed, not built)

**2026-10-07: no base, no codextras** (owner: "there's no reason to
create a language on this base. So, could we remove it, and let Signatar
be the lowest level?", then "`a`" to a core taking powers away). Signatar
is the language and the lowest level, and does what the base did.
`DECISIONS.md` 2026-10-07 has it; older passages quoting the base keep
their words. **Later that day: Signatar *is* the base, "more like the
JVM", and a new category, the dialect, ranks between it and cores and
brings the syntax** (owner: "the syntax should come from a dialect");
**the official dialect is CyborgPL**; Limbs in different dialects do not
use each other (owner: "`b`"). **A core brings a feature the base
understands, which any dialect may spell, and may bring its own words for
the dialects it names** (owner: "C is great"); MTM and WSTN are such cores,
bringing words to CyborgPL. Where a dialect takes away a power a core
grants, the dialect wins (design claude's reading).

The owner: "I was thinking of making the base lang more lower-level ... I
wanted to move things like, plus, div, you know to std. And even more",
then: "Just low level as fuck, even if it then requires another category
that has more authority than a core." This note is the picture;
`DECISIONS.md` has each step with the owner's words. Spellings marked
*sketch* are design claude's and not ruled.

The owner's reasons: "it's fun, we'll eventually need the no_std
equivalent anyways, and other stuff". The theme was planned: a cyborg
"can't do shit without a core or limbs" (owner).

## The stack

```
Signatar    the base, "more like the JVM": the meaning (types, ownership,
            Precog, powers); reads the files, loads the dialect and cores,
            enforces the grants, hands LLVM IR to LLVM
CyborgPL    the official dialect: the syntax; may take powers away and
            require or forbid cores (design claude's reading)
IEG         the official core: raw operations, inline.LL, the !check labels
ASP         numbers mean their true values; to. conversions; run-time checks
Genny       generics and interfaces
MCPS        parallel loops and concur (needs threads)
AutoGPU     automatic offload on every GPU; Metal, Nvidia and Vulkan
            cores for hand-written GPU code
std         a limb with powers: every operator, logic, text, printing
your limbs  everything else
```

When people say "CyborgPL" they mean Signatar with whatever cores, "like
how they say BIOS for UEFI" (owner); the website will say **Signatar
CyborgPL** when this lands, "Or else, it would just be an Assembly
monster" (owner).

## The three kinds of thing

| | What it is | Authority | Written in |
|---|---|---|---|
| **Signatar** | the language: syntax, types, its own rules, its checker, and the loading and grant enforcement the base did | highest: defines which powers exist and limits cores | (CCA itself) |
| **A core** | "kinda more of like a compiler mod" (owner) | grants powers to limbs, and may take them away (2026-10-07); may add syntax | Rust plugin + CyborgPL |
| **A limb** | a library | only the powers granted to it | CyborgPL |

- **Cores make dialects**, which are still CyborgPL. A restricted
  dialect, a game engine's say, is a core that takes powers away from the
  programs that list it (owner, 2026-10-07: "`a`"). Until 2026-10-07 a
  codextra was a category, another language on the base ("this shit could
  make another Codextra be another lang", owner); there is no base now.
- **Core is a category too**: anyone may write one, cores depend on cores,
  and an unverified core is used with a warning, never refused ("Maybe
  it's their own, maybe it's their friend's", owner). A core need not put
  its name in the syntax.
- **The warning** for one nobody verified (owner, 2026-10-06):

  ```
  warning: the core "Fastmath" (registry ID 7f3a…) is not verified
    where: CCA.cplr, cores
    note:  a core runs inside the compiler and can change what your
           program does. Use it only if you trust where it came from.
  ```

- **Trust**: Signatar and the official cores are verified (RRx2 checks
  IEG's IR against its contracts); installing any other is already
  trusting it, as installing any program is.

**IDs** (owner, 2026-10-06): a core is listed by its registry ID and
version, `cca::Core[id = "…", version = "…"]`. The official ones, first
version `2026-10-06T18:04:29+07:00`:

```
IEG      5d0e846b2a0f57f6fae20216e8d856d41b6257e0a6104aaf0bc55b57b44b4085
ASP      4882d1f3bf7b11d59879e4921672f15e91369e25d55a10e3dcd24e9b3fbe676f
Genny    c3e299d99a13567b2039c13fc17d0ca562be9d6f657f8c40a4774081983b20c9
MCPS     7d0ccfe6e41eed3a4d8ffe00ccaada9b89a4b9a3ea553a9d869cb6902e2ca75d
AutoGPU  67b2514aab0ffa5ca9c3ae48144498880466d68aa3b8b8f8e1082e6032c24c4d
Metal    3e7b81fec072c86d58538d10978aff1d42814c9e907f1259766851795cded9b7
Nvidia   beefcce9ed7cccf0d62f5a6e004c4bd53c6ae9e3a45c48883d3b1dc4be913361
Vulkan   63ff7ed8da5e0eead0164488e89553b240501f16d4be863ba6be1e46b51ec3ec
WSTN     e86b0df56e35b02b2ee91df6d89e63011564364d404ed7f5ea25d768d6d854d4  (first version 2026-10-06T18:48:22+07:00)
MTM      9c6c773027ae873a736d2e9b2dc398fd4807a7df6cf43b7e233c3447af3d6a56  (first version 2026-10-06T19:18:54+07:00)
std      fff4302dffc4a894a2bab6baf724eb21dfc05a2ad9818509d8001013d6761936  (a limb, 2026-10-06T12:57:14+07:00)
```

## Signatar, the language

"Signature but cooler" (owner). Very low level: "not nothing, but it's
pretty lower-level (say writing fucking Assembly for a lot of things)"
(owner). It holds:

- **syntax**, and what the base did: reading the files, loading the cores,
  enforcing the grants, handing its LLVM IR to LLVM;
- **types, `let`, functions, `if`, loops, `match`, structs, enums, `bool`**;
- **the borrow rules**, lifetime groups (`@x`) included, and **Precog**
  (it proves things about Signatar's own rules; at Signatar alone that is
  mostly borrows, types and shape, since its maths is assembly);
- **SIMD**, `and.then` / `or.else` (they skip their right side, which an
  operator from std cannot), **`on.target`** (below);
- **fixed arrays** (`array[T]`) and **text literals**, which need no heap;
- **raw numbers**: literals are bit patterns, as the machine's (an `int8`
  holding `128` holds -128's bits), with no negative literals; a literal
  wider than its type's bits (`256` in an `int8`) is refused (owner,
  2026-10-06);
- **kernel-grade raw memory**, beside ownership: pointers as values,
  addresses as numbers and back, volatile access, memory with no owner;
- **no operators, no `to.` conversions, no markers**: in Signatar alone, a
  negation or a reinterpretation is assembly.

## The official cores

- **IEG, the Intense Energy Generator.** Raw operations as plain calls
  (`add`, `negate`, `compare` *sketch*), with contracts Precog trusts;
  `inline.LL { … }`, LLVM IR in place; and the **labels**: `!check`,
  `NO_FUCKING_CHECKS`, and the rule that raw memory and unproven code sit
  under `!check`, so a kernel on Signatar + IEG has clear syntax ("like
  Rust's kernel mode also has clear syntax", owner). Not fully unsafe.
  It also brings **`on.machine`**, the startup check of the chip's
  features.
- **ASP, "Abit Safer, Please".** Numbers mean their true values: literals
  as values (`200` is two hundred, refused in an `int8`), exact in between
  and checked where they land, the middle limit (the widest operand's
  width, at least 64 bits), and `check` with the run-time checks. It
  brings the **`to.` syntax**: checked conversions and the raw
  `to.int8.bits[128]`, never the default.
- **Genny.** Generics (`<T>`, bounds, `and`, `any`), interfaces and
  `satisfies`. Needs only Signatar, so a kernel may take it; zero cost at
  run time, paid in program size and build time.
- **MCPS, "More Concurrently Parallel Shit".** Splitting proven loops
  across cores, and `concur`, channels, `whichever`, timers, sharing one
  thread pool. A program without it (a kernel) runs its loops in order and
  `for para` is refused. Needs IEG, not std.
- **WSTN, "Wanted Some Thai Names"** (owner, 2026-10-06): names in every
  script. Signatar's names, bare and quoted, are English letters, digits
  and `_`; WSTN opens them to every script and to emoji (owner: "`b`").
- **MTM, "Maybe Too Much"** (owner, 2026-10-06): other spellings, so
  each layer keeps one, each working only where its owner is listed:
  `immut` (Signatar's `!mut`), `nocheck` (IEG's `!check`), `not` (std's
  `! 'x'`), and for Signatar: The arrows `->`,
  `>>`, `>>>`, `<-`, `<<`, `<<<`, `<->`, `<<>>`, `<<<>>>`, `<---->`, `<--|-->`,
  `<-|->`, `<<|>>`, `<<<|>>>` (Signatar: `-->`, `<--`, `<-->`); `&` / `&mut`;
  Rust's number names; `bfloat16` / `bf16`; `!std.i24`; the short names
  for an `inline.` language.
- **GPU**: **AutoGPU**, automatic offload on every GPU (it translates
  plain `for` loops), and three cores for hand-written GPU code: **Metal**
  (`inline.MSL`), **Nvidia** and **Vulkan** (AMD, NVIDIA, Intel, Android).
  Listing all runs on all; listing one is a choice, as Valorant chose
  Windows (owner). Core names are not trademarked; the categories and
  names like Signatar are (owner).

## std, a limb with powers

**std needs ASP and IEG** (owner, 2026-10-06): a manifest listing std
without ASP is refused, so std's maths always has true values and checks,
and without IEG too, which grants std its operators.

Written in CyborgPL, calling the Rust runtime for what CyborgPL cannot
express yet. It holds **every operator**: `plus`, `minus`, `times`,
`div.X` / `mod.X`, `pow`, comparisons, `and`, `or`, `not`, `not.bit`, `-`
(so `-5` is std's `-` applied to `5`), bitwise and `++`; the text and
decimal conversions it adds into ASP's `to.`; printing; friendly names
for LLVM operations (`std::bits.count_ones` *sketch*); the everyday
interfaces (`Number`, `Ordered`); and what needs a heap: growing
**lists**, **maps**, and **text operations** (`str.bytes`, `++`). A kernel
makes its own lists and allocator, as Linux does.

## Powers and who grants them

- **Signatar defines which powers exist** and limits cores (no core
  may switch off Signatar's borrow rules).
- **Cores grant powers to limbs**: to every limb built with them, and
  extra to favourites named by registry ID (names repeat, IDs do not).
  **A core may also take powers away** from the programs and limbs built
  with it (owner, 2026-10-07: "`a`"), so a game engine's core makes a
  restricted dialect its store can require; how a core spells it, and
  which wins when one core grants what another takes away, are open.
- **A limb lists the powers it uses** in its manifest; each works only if
  a core it is built for granted it. std's: operators on built-in types
  and prefix operators (from IEG), adding conversions (from ASP), and
  plugging its list into `for`, a power IEG may grant every limb, so a
  kernel's own list works with `for` too.
- **The program may take powers away, never add** (`deny`), and **an
  upgrade that starts using a new power is flagged**.
- **The dangerous abilities are powers too** (owner, 2026-10-07), which
  Signatar grants every limb by default: `inline_assembly`, `c_calls`,
  `raw_memory`. **Cores may add powers of their own.** A power narrows with
  `.` and names exact things in `[ ]`; `deny` takes `.except[ … ]` and
  `.for[ … ]`: `deny = [cca::Power.c_calls.except["sqrt"],
  cca::Power.inline_assembly.for["fastjson"],]`.

## Manifests

`CCA.cplr` is one real CyborgPL value, not named `const`s ("if the syntax
looks real, but it's not real. It feels like a child roleplaying", owner):

```
cca::Manifest[
    name = "shop",
    prefix = "shop",
    cores = [cca::Core[id = "5d0e846b…44b4085", version = "2026-10-06T18:04:29+07:00"],],   # IEG
    dependencies = [cca::Dep[path = "std"],],
    powers = [cca::Power.operators_on_builtin_types,],
    deny = [cca::Power.inline_assembly,],
];
```

`Synthia.ttsr` has the same as sections (`[my cores]` *sketch*).

## Installing

The website offers packages: Signatar; + IEG; + ASP +
Genny + std (std needs ASP; owner, 2026-10-06); all, with source
(owner). Packages with std set things up so `cca new`
lists std (and ASP, Genny), std by its registry ID and version like
any limb, CCA using the installed copy when it matches (owner, 2026-10-06;
std's ID is `fff4302dffc4a894a2bab6baf724eb21dfc05a2ad9818509d8001013d6761936`; its version is the release's date and time, ISO 8601 with
the releaser's offset, first `2026-10-06T12:57:14+07:00`); without std, `cca new` writes an empty
manifest. A `.cpls` file built with no `CCA.cplr` beside it gets what
the machine's package installed (owner, 2026-10-06). The installer writes
that down in **a machine-wide record** (owner: "Good idea"), which `cca
new` reads too; its name and
lives where each OS keeps such files (owner; macOS `~/Library/Application
Support/CyborgPL/`, Linux `~/.local/share/cyborgpl/`, Windows
`%APPDATA%\CyborgPL\`), named **`installed.cplr`** (owner); its value
is *sketch*: `cca::Installed[ … ]`. **There is an uninstaller**
(owner): with nothing named it removes everything the installer put
there, with a piece named (std, say) only that piece; never a project. It
comes with Synthia only, which can uninstall itself too (owner); a
CCA-only user removes CyborgPL by hand, "just like how a CCA user would
have to change a limb's prefix themselves" (owner).

## Code for one machine

Chosen **when compiling**, with `else` for the rest:

```
on.target.chip aarch64 { !check inline.ASM { neg x0, x0 } with [ … ]; }
else on.target.chip x86_64 { !check inline.ASM { neg rax } with [ … ]; }
else { let 'y' := -'x'; }
on.target.os macos 15 { … }          # a minimum: refuses to start on older
on.target aarch64-macos { … }        # one exact target
```

and **when running**, checking the actual machine once at startup:
`on.machine.chip sve { … } else { … }`, so one build runs everywhere and is
fast where it can be.

## Hello World at every level

Signatar alone: raw system calls in `inline.ASM`; Signatar through C:
`extern.C fn write`; inside IEG: `inline.LL`; a program on IEG with no
std: IEG's raw write; with std: `std::print.useful.stdout["Hello, world!",
\n];`; and a limb of your own on top. The lowest level is Signatar's raw
assembly; there is no base below it now.

## Open

- The stable **plugin interface** for cores: compiler
  claude proposes it when building, the owner approves (2026-10-06).
- **Building it**: the oracle reads `cca::Manifest` with `cores`, and a
  Limb whose manifest does not list std reaches no operator and no
  `std::`, nor a core's words when it does not list that core
  (2026-10-06), and without ASP a whole-number literal lands as a bit
  pattern; lists, maps and text need std; ASP's `to.int8.bits[…]`;
  `on.target` and `on.machine` (minimum versions not built: how a build
  names its oldest OS and chip features is open). Not built: IEG's raw
  operations, the powers, cores as plugins, the installer.
