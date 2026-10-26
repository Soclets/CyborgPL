# the base, limbs and kits (owner, 2026-10-05/06/07; being designed, not built)

> Names (owner, 2026-10-09): **Signatar is CyborgPL's base, the base is
> CyborgPL's.** Signatar (2026-10-05) is the slim base this note describes, as of
> 2026-10-07: no operators, its maths in assembly. On 2026-10-08 the base was
> made practical (operators, true-value numbers, ASP and Genny folded in,
> iterators, `optional`; `DECISIONS.md`, "The base should be practical"), and
> that is the base, **built in the oracle 2026-10-09**. the base has
> `plus`; Signatar did not. The limbs ASP and Genny are retired. Below,
> "Signatar" is the 2026-10-07 slim base, kept as history.

> Revised 2026-10-07: code here is in the new syntax; an old spelling in the prose is history. The old → new table is `revision-2026-10-07.md`.

**2026-10-07: no base, no codextras** (owner: "there's no reason to
create a language on this base. So, could we remove it, and let Signatar
be the lowest level?", then "`a`" to a core taking powers away). the base
is the language and the lowest level, and does what the base did.
`DECISIONS.md` 2026-10-07 has it; older passages quoting the base keep
their words. Later that day a **dialect** layer (syntax above the base, "more like the
JVM") was added and then **dropped** (owner: "just let them fork so they
could do whatever they want", then "Yeah"): the language is the lowest
level, limbs customize (and may take powers away), and anything deeper,
its own syntax or optimizer, is a fork.

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
the base    the language itself, the lowest level: reads the files, loads
            the cores, enforces the grants, hands its LLVM IR to LLVM
IEG         the official core: raw operations, inline.LL, the no_check labels
ASP         numbers mean their true values; to. conversions; run-time checks
Genny       generics and interfaces
MCPS        parallel loops and concur (needs threads)
AutoGPU     automatic offload on every GPU; Metal, Nvidia and Vulkan
            cores for hand-written GPU code
std         a kit with powers: every operator, logic, text, printing
your kits  everything else
```

When people say "CyborgPL" they mean the base with whatever limbs, "like
how they say BIOS for UEFI" (owner); the website will say **the base
CyborgPL** when this lands, "Or else, it would just be an Assembly
monster" (owner).

## The three kinds of thing

| | What it is | Authority | Written in |
|---|---|---|---|
| **the base** | the language: syntax, types, its own rules, its checker, and the loading and grant enforcement the base did | highest: defines which powers exist and limits limbs | (kayo itself) |
| **A limb** | "kinda more of like a compiler mod" (owner) | grants powers to kits, and may take them away (2026-10-07); may add syntax | Rust limb + CyborgPL |
| **A kit** | a library | only the powers granted to it | CyborgPL |

- **Limbs make dialects**, which are still CyborgPL. A restricted
  dialect, a game engine's say, is a limb that takes powers away from the
  programs that list it (owner, 2026-10-07: "`a`"). Until 2026-10-07 a
  codextra was a category, another language on the base ("this shit could
  make another Codextra be another lang", owner); there is no base now.
- **Limb is a category too**: anyone may write one, limbs depend on limbs,
  and an unverified limb is used with a warning, never refused ("Maybe
  it's their own, maybe it's their friend's", owner). A limb need not put
  its name in the syntax.
- **The warning** for one nobody verified (owner, 2026-10-06):

  ```
  warning: the limb "Fastmath" (registry ID 7f3a…) is not verified
    where: CyJet.cyr, limbs
    note:  a limb runs inside the compiler and can change what your
           program does. Use it only if you trust where it came from.
  ```

- **Trust**: the base and the official limbs are verified (RRx2 checks
  IEG's IR against its contracts); installing any other is already
  trusting it, as installing any program is.

**IDs** (owner, 2026-10-06): a limb is listed by its registry ID and
version, `cyjet::limb[id = "…", version = "…"]`. The official ones, first
version `2026-10-06T18:04:29+07:00`:

```
IEG      5d0e846b2a0f57f6fae20216e8d856d41b6257e0a6104aaf0bc55b57b44b4085  (dissolved into the base, 2026-10-12)
ASP      4882d1f3bf7b11d59879e4921672f15e91369e25d55a10e3dcd24e9b3fbe676f
Genny    c3e299d99a13567b2039c13fc17d0ca562be9d6f657f8c40a4774081983b20c9
MCPS     7d0ccfe6e41eed3a4d8ffe00ccaada9b89a4b9a3ea553a9d869cb6902e2ca75d
AutoGPU  67b2514aab0ffa5ca9c3ae48144498880466d68aa3b8b8f8e1082e6032c24c4d
Metal    3e7b81fec072c86d58538d10978aff1d42814c9e907f1259766851795cded9b7
Nvidia   beefcce9ed7cccf0d62f5a6e004c4bd53c6ae9e3a45c48883d3b1dc4be913361
Vulkan   63ff7ed8da5e0eead0164488e89553b240501f16d4be863ba6be1e46b51ec3ec
WSTN     e86b0df56e35b02b2ee91df6d89e63011564364d404ed7f5ea25d768d6d854d4  (first version 2026-10-06T18:48:22+07:00)  (part of unicode, 2026-10-12)
MTM      9c6c773027ae873a736d2e9b2dc398fd4807a7df6cf43b7e233c3447af3d6a56  (first version 2026-10-06T19:18:54+07:00)  (gone, 2026-10-07)
TAS      63baa4cf89d8b6df6adceb812df005b30ea2771fd56f2117d0246ef911de21aa  (first version 2026-10-07T18:08:59+07:00)  (dissolved into the base, 2026-10-12)
AI       8c723ccae25a719cd66eb44e95aeeef0557acec1467624ab91092c4ada766f21  (first version 2026-10-07T20:22:24+07:00)
WSMW     0fb49902b57b8bc0b72caf424d80e17d035038e2cac58292d92f18b792d205a0  (first version 2026-10-07T20:22:24+07:00)
Money    d7b62f4448a5b23d792a25bcdba89220e02691fb5e27cf18640ee491d049df0f  (first version 2026-10-07T20:22:24+07:00)
UniTect  137a9a35afc69ec6ee796bc39eeefc75680db17ce8a04443126a492812a0ae96  (first version 2026-10-07T20:22:24+07:00)  (part of unicode, 2026-10-12)
CAPS     90d0813dc72a0c7d54a4ded7a2c9577e77f0036958b245ab01808f9a8568bd01  (first version 2026-10-07T20:22:24+07:00)  (part of unicode, 2026-10-12)
CNC      04b99a7cbb141b84b9f2f38c085c124e2badc37d663f71c6deebdd6ac24e79b1  (first version 2026-10-10T16:42:00+07:00)  ("CNC Not C": everything about C)
unicode    55a2e2de2eca2e06e8a4b3aa2c5ef7245f3f55d4dfbfd872a3ddd9500f74dece  (first version 2026-10-12T10:19:44+07:00)  (names and text in every human language: WSTN, UniTect and CAPS, retired, 2026-10-12)
std      fff4302dffc4a894a2bab6baf724eb21dfc05a2ad9818509d8001013d6761936  (a kit, 2026-10-06T12:57:14+07:00)
```

## Signatar, the language as of 2026-10-07 (superseded by the base, 2026-10-08)

"Signature but cooler" (owner). Very low level: "not nothing, but it's
pretty lower-level (say writing fucking Assembly for a lot of things)"
(owner). It holds:

- **syntax**, and what the base did: reading the files, loading the limbs,
  enforcing the grants, handing its LLVM IR to LLVM;
- **types, `let`, functions, `if`, loops, `match`, structs, enums, `bool`**;
- **the borrow rules**, lifetime groups (`@x`) included, and **the prover**
  (it proves things about Signatar's own rules; at Signatar alone that is
  mostly borrows, types and shape, since its maths is assembly);
- **SIMD**, `and.then` / `or.else` (they skip their right side, which an
  operator from std cannot), **`on.target`** (below);
- **fixed arrays** (`array[T]`) and **text literals**, which need no heap;
- **raw numbers**: literals are bit patterns, as the machine's (an `i8`
  holding `128` holds -128's bits), with no negative literals; a literal
  wider than its type's bits (`256` in an `i8`) is refused (owner,
  2026-10-06);
- **kernel-grade raw memory**, beside ownership: pointers as values,
  addresses as numbers and back, volatile access, memory with no owner;
- **no operators, no `to.` conversions, no markers**: in Signatar alone, a
  negation or a reinterpretation is assembly.

## The limb mark, `%` (owner, 2026-10-26; not built)

**Every piece a limb adds is marked `%`; the base is bare; a kit is its path** (owner: "If kits already have marks, why
not limbs. And then, lang could be the absence of such"). The mark goes on the first word that is the limb's (design
claude's reading of the owner's examples):

```
%extern.C fn puts [$s: ptr[u8]] -> %C.int;             # CNC
incl.%C["sdl.h"];                                      # `incl` is the base's
let $price: %d64 = 19.50;                              # Money
let $odd: %not_std.i24 = 5;                            # WSMW: `not_std` is only its
let $name: str.utf8[max: 40 %graphemes] = "café";      # unicode
%concur with [lend $ch] { %channel.send(lend $ch, value: 1); }   # MCPS
no_check inline.%MSL threads 64 { … } with [lend.mut $xs -> buffer(0)];   # Metal
```

So `%` no longer means percent: `80 percent`, `percent[d32]`. A program's limbs show where they are used, and `%C`
nowhere means it never calls C. Open: names in other scripts; the `%` old-spelling refusal for `mod`.

## The official limbs

- **CNC, "CNC Not C"** (owner, 2026-10-10): everything about C, one limb for
  that expertise: `extern.C`, `incl.C`, `layout.C`, callbacks to C, the
  `C.` types (`C.int`, `C.long`, `C.long_double`, …), and the `c_calls`
  power. The base keeps none of it, but keeps `ptr` and inline
  assembly, which are the machine's, not C's. A kit that does not list
  CNC provably never calls C. It brings Clang, which reads C headers:
  borrowed from the machine until the release, then shipped inside CNC
  (owner, 2026-10-10). Limbs are being regrouped by expertise
  (owner: "Like a big one for a expertise"); the limbs below predate that.

- **IEG, the Intense Energy Generator.** Raw operations as plain calls
  (`add`, `negate`, `compare` *sketch*), with contracts the prover trusts;
  `inline.LL { … }`, LLVM IR in place; and the **labels**: `no_check`,
  `NO_FUCKING_CHECKS`, and the rule that raw memory and unproven code sit
  under `no_check`, so a kernel on the base + IEG has clear syntax ("like
  Rust's kernel mode also has clear syntax", owner). Not fully unsafe.
  It also brings **`on.machine`**, the startup check of the chip's
  features.
- **ASP, "Abit Safer, Please".** Numbers mean their true values: literals
  as values (`200` is two hundred, refused in an `i8`), exact in between
  and checked where they land, the middle limit (the widest operand's
  width, at least 64 bits), and `check` with the run-time checks. It
  brings the **`to.` syntax**: checked conversions and the raw
  `to.i8.bits[128]`, never the default.
- **Genny.** Generics (`<T>`, bounds, `and`, `any`), interfaces and
  `satisfies`. Needs only the base, so a kernel may take it; zero cost at
  run time, paid in program size and build time.
- **MCPS, "More Concurrently Parallel Shit".** Splitting proven loops
  across cores, and `concur`, channels, `whichever`, timers, sharing one
  thread pool. A program without it (a kernel) runs its loops in order and
  `for para` is refused. Needs IEG, not std.
- **WSTN, "Wanted Some Thai Names"** (owner, 2026-10-06): names in every
  script. the base's names, bare and quoted, are English letters, digits
  and `_`; WSTN opens them to every script and to emoji (owner: "`b`").
- **MTM, "Maybe Too Much"** (owner, 2026-10-06): other spellings, so
  each layer keeps one, each working only where its owner is listed:
  `immut` (the base's `!mut`), `nocheck` (IEG's `!check`), `not` (std's
  `! 'x'`), and for the base: The arrows `->`,
  `>>`, `>>>`, `<-`, `<<`, `<<<`, `<->`, `<<>>`, `<<<>>>`, `<---->`, `<--|-->`,
  `<-|->`, `<<|>>`, `<<<|>>>` (the base: `-->`, `<--`, `<-->`); `&` / `&mut`;
  Rust's number names; `bfloat16` / `bf16`; `!std.i24`; the short names
  for an `inline.` language.
- **GPU**: **AutoGPU**, automatic offload on every GPU (it translates
  plain `for` loops), and three limbs for hand-written GPU code: **Metal**
  (`inline.MSL`), **Nvidia** and **Vulkan** (AMD, NVIDIA, Intel, Android).
  Listing all runs on all; listing one is a choice, as Valorant chose
  Windows (owner). Limb names are not trademarked; the categories and
  names like the base are (owner).

## std, a kit with powers

**std needs ASP and IEG** (owner, 2026-10-06): a manifest listing std
without ASP is refused, so std's maths always has true values and checks,
and without IEG too, which grants std its operators.

Written in CyborgPL, calling the runtime, the Rust runtime, for what CyborgPL cannot
express yet. It holds **every operator**: `plus`, `minus`, `times`,
`div.X` / `mod.X`, `pow`, comparisons, `and`, `or`, `not`, `not.bit`, `-`
(so `-5` is std's `-` applied to `5`), bitwise and `++`; the text and
decimal conversions it adds into ASP's `to.`; printing; friendly names
for LLVM operations (`std::bits.count_ones` *sketch*); the everyday
interfaces (`Number`, `Ordered`); and what needs a heap: growing
**lists**, **maps**, and **text operations** (`str.bytes`, `++`). A kernel
makes its own lists and allocator, as Linux does.

## Powers and who grants them

- **the base defines which powers exist** and limits limbs (no limb
  may switch off the base's borrow rules).
- **Limbs grant powers to kits**: to every kit built with them, and
  extra to favourites named by registry ID (names repeat, IDs do not).
  **A limb may also take powers away** from the programs and kits built
  with it (owner, 2026-10-07: "`a`"), so a game engine's limb makes a
  restricted dialect its store can require; how a limb spells it, and
  which wins when one limb grants what another takes away, are open.
- **A kit lists the powers it uses** in its manifest; each works only if
  a limb it is built for granted it. std's: operators on built-in types
  and prefix operators (from IEG), adding conversions (from ASP), and
  plugging its list into `for`, a power IEG may grant every kit, so a
  kernel's own list works with `for` too.
- **The program may take powers away, never add** (`deny`), and **an
  upgrade that starts using a new power is flagged**.
- **The dangerous abilities are powers too** (owner, 2026-10-07), which
  the base grants every kit by default: `inline_assembly` and
  `raw_memory`; `c_calls` is CNC's since 2026-10-10, granted to a kit
  that lists CNC (design claude's reading: the base keeps nothing of C). **Limbs may add powers of their own.** A power narrows with
  `.` and names exact things in `[ ]`; `deny` takes `.except[ … ]` and
  `.for[ … ]`: `deny = [cyjet::power.c_calls.except["sqrt"],
  cyjet::power.inline_assembly.for["fastjson"],]`.

## Manifests

`CyJet.cyr` is one real CyborgPL value, not named `const`s ("if the syntax
looks real, but it's not real. It feels like a child roleplaying", owner):

```
cyjet::manifest[
    name = "shop",
    prefix = "shop",
    limbs = [cyjet::limb[id = "5d0e846b…44b4085", version = "2026-10-06T18:04:29+07:00"],],   # IEG
    dependencies = [cyjet::dep.path["std"],],
    powers = [cyjet::power.operators_on_builtin_types,],
    deny = [cyjet::power.inline_assembly,],
];
```

`Synthia.ttsr` has the same as sections (`[my cores]` *sketch*).

## Installing

The website offers packages: the base; + IEG; + ASP +
Genny + std (std needs ASP; owner, 2026-10-06); all, with source
(owner). Packages with std set things up so `synapse new`
lists std (and ASP, Genny), std by its registry ID and version like
any kit, kayo using the installed copy when it matches (owner, 2026-10-06;
std's ID is `fff4302dffc4a894a2bab6baf724eb21dfc05a2ad9818509d8001013d6761936`; its version is the release's date and time, ISO 8601 with
the releaser's offset, first `2026-10-06T12:57:14+07:00`); without std, `synapse new` writes an empty
manifest. A `.cy` file built with no `CyJet.cyr` beside it gets what
the machine's package installed (owner, 2026-10-06). The installer writes
that down in **a machine-wide record** (owner: "Good idea"), which `cca
new` reads too; its name and
lives where each OS keeps such files (owner; macOS `~/Library/Application
Support/CyborgPL/`, Linux `~/.local/share/cyborgpl/`, Windows
`%APPDATA%\CyborgPL\`), named **`installed.cyr`** (owner); its value
is *sketch*: `cyjet::installed[ … ]`. **There is an uninstaller**
(owner): with nothing named it removes everything the installer put
there, with a piece named (std, say) only that piece; never a project. It
comes with Synthia only, which can uninstall itself too (owner); a
kayo-only user removes CyborgPL by hand, "just like how a Zomp user would
have to change a strike's prefix themselves" (owner).

## Code for one machine

Chosen **when compiling**, with `else` for the rest:

```
on.target.chip aarch64 { no_check inline.ASM { neg x0, x0 } with [ … ]; }
else on.target.chip x86_64 { no_check inline.ASM { neg rax } with [ … ]; }
else { let $y := -$x; }
on.target.os macos 15 { … }          # a minimum: refuses to start on older
on.target aarch64-macos { … }        # one exact target
```

and **when running**, checking the actual machine once at startup:
`on.machine.chip sve { … } else { … }`, so one build runs everywhere and is
fast where it can be.

## Hello World at every level

the base alone: raw system calls in `inline.ASM`; the base through C:
`extern.C fn write`; inside IEG: `inline.LL`; a program on IEG with no
std: IEG's raw write; with std: `std::print.useful.stdout["Hello, world!",
\n];`; and a kit of your own on top. The lowest level is the base's raw
assembly; there is no base below it now.

## Open

- The stable **limb interface** for limbs: compiler
  claude proposes it when building, the owner approves (2026-10-06).
- **Building it**: the oracle reads `cyjet::manifest` with `cores`, and a
  kit whose manifest does not list std reaches no operator and no
  `std::`, nor a limb's words when it does not list that limb
  (2026-10-06), and without ASP a whole-number literal lands as a bit
  pattern; lists, maps and text need std; ASP's `to.i8.bits[…]`;
  `on.target` and `on.machine` (minimum versions not built: how a build
  names its oldest OS and chip features is open). Not built: IEG's raw
  operations, the powers, limbs as limbs, the installer.
