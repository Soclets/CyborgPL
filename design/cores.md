# Base, codextras, cores and limbs (owner, 2026-10-05/06; being designed, not built)

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
base        the ignition: reads files, loads the codextra and cores, enforces
            grants, hands the codextra's LLVM IR to LLVM unread; no syntax,
            nothing to write in
Signatar    the official codextra: the language itself, very low level
IEG         the official core: raw operations, inline.LLVM, the nocheck labels
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

## The four kinds of thing

| | What it is | Authority | Written in |
|---|---|---|---|
| **The base** | the ignition, "basically nothing" (owner) | enforces everyone's grants | (CCA itself) |
| **A codextra** | a language: syntax, types, its own rules, its checker; hands the base LLVM IR | highest: defines which powers exist and limits cores | Rust plugin + CyborgPL |
| **A core** | "kinda more of like a compiler mod" (owner) | grants powers to limbs; may add syntax | Rust plugin + CyborgPL |
| **A limb** | a library | only the powers granted to it | CyborgPL |

- **Codextra is a category.** Another codextra is another language on
  the CyborgPL base ("this shit could make another Codextra be another
  lang", owner). Lilith is not one: "That shit ain't in the CyborgPL
  family" (owner). **Cores make dialects**, which are still CyborgPL.
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

  and the same with "codextra" for a codextra.
- **Trust**: the official codextra and cores are verified (RRx2 checks
  IEG's IR against its contracts); installing any other is already
  trusting it, as installing any program is.

## Signatar, the official codextra

"Signature but cooler" (owner). Very low level: "not nothing, but it's
pretty lower-level (say writing fucking Assembly for a lot of things)"
(owner). It holds:

- **syntax**: a codextra brings its own parser, so the base does not know
  what a quoted name is;
- **types, `let`, functions, `if`, loops, `match`, structs, enums, `bool`**;
- **the borrow rules**, lifetime groups (`@x`) included, and **Precog**
  (it proves things about Signatar's own rules; at Signatar alone that is
  mostly borrows, types and shape, since its maths is assembly);
- **SIMD**, `and.then` / `or.else` (they skip their right side, which an
  operator from std cannot), **`on.target`** (below);
- **fixed arrays** (`array[T]`) and **text literals**, which need no heap;
- **raw numbers**: literals are bit patterns, as the machine's (an `int8`
  holding `128` holds -128's bits), with no negative literals;
- **kernel-grade raw memory**, beside ownership: pointers as values,
  addresses as numbers and back, volatile access, memory with no owner;
- **no operators, no `to.` conversions, no markers**: in Signatar alone, a
  negation or a reinterpretation is assembly.

## The official cores

- **IEG, the Intense Energy Generator.** Raw operations as plain calls
  (`add`, `negate`, `compare` *sketch*), with contracts Precog trusts;
  `inline.LLVM { … }`, LLVM IR in place; and the **labels**: `nocheck`,
  `NO_FUCKING_CHECKS`, and the rule that raw memory and unproven code sit
  under `nocheck`, so a kernel on Signatar + IEG has clear syntax ("like
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
- **GPU**: **AutoGPU**, automatic offload on every GPU (it translates
  plain `for` loops), and three cores for hand-written GPU code: **Metal**
  (`inline.MSL`), **Nvidia** and **Vulkan** (AMD, NVIDIA, Intel, Android).
  Listing all runs on all; listing one is a choice, as Valorant chose
  Windows (owner). Core names are not trademarked; the categories and
  names like Signatar are (owner).

## std, a limb with powers

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

- **The codextra defines which powers exist** and limits cores (no core
  may switch off Signatar's borrow rules).
- **Cores grant powers to limbs**: to every limb built with them, and
  extra to favourites named by registry ID (names repeat, IDs do not).
- **A limb lists the powers it uses** in its manifest; each works only if
  a core it is built for granted it. std's: operators on built-in types
  and prefix operators (from IEG), adding conversions (from ASP), and
  plugging its list into `for`, a power IEG may grant every limb, so a
  kernel's own list works with `for` too.
- **The program may take powers away, never add** (`deny`), and **an
  upgrade that starts using a new power is flagged**.

## Manifests

`CCA.cplr` is one real CyborgPL value, not named `const`s ("if the syntax
looks real, but it's not real. It feels like a child roleplaying", owner):

```
cca::Manifest[
    name = "shop",
    prefix = "shop",
    cores = [cca::Core[name = "IEG"],],
    dependencies = [cca::Dep[path = "std"],],
    powers = [cca::Power.operators_on_builtin_types,],
    deny = [cca::Power.inline_asm,],
];
```

`Synthia.ttsr` has the same as sections (`[my cores]` *sketch*).

## Installing

The website offers packages: base; base + Signatar; + IEG; + std; all,
with source (owner). Signatar installs itself as the machine's default
codextra, which the base uses to read `CCA.cplr`; another codextra's
installer overrides it. Packages with std set things up so `cca new`
lists std (and ASP, Genny); without std, `cca new` writes an empty
manifest. A `.cpls` file built with no `CCA.cplr` beside it gets what
the machine's package installed (owner, 2026-10-06). The installer writes
that down in **a machine-wide record** (owner: "Good idea"), which `cca
new` reads too and another codextra's installer rewrites; its name and
lives where each OS keeps such files (owner; macOS `~/Library/Application
Support/CyborgPL/`, Linux `~/.local/share/cyborgpl/`, Windows
`%APPDATA%\CyborgPL\`), named **`installed.cplr`** (owner); its value
is *sketch*: `cca::Installed[ … ]`. **There is an uninstaller**
(owner): with nothing named it removes everything the installer put
there, with a piece named (std, say) only that piece; never a project. It
comes with Synthia, which can uninstall itself too (owner: "both").

## Code for one machine

Chosen **when compiling**, with `else` for the rest:

```
on.target.chip aarch64 { nocheck inline.S { neg x0, x0 } with [ … ]; }
else on.target.chip x86_64 { nocheck inline.S { neg rax } with [ … ]; }
else { let 'y' := -'x'; }
on.target.os macos 15 { … }          # a minimum: refuses to start on older
on.target aarch64-macos { … }        # one exact target
```

and **when running**, checking the actual machine once at startup:
`on.machine.chip sve { … } else { … }`, so one build runs everywhere and is
fast where it can be.

## Hello World at every level

Signatar alone: raw system calls in `inline.S`; Signatar through C:
`extern.C fn write`; inside IEG: `inline.LLVM`; a program on IEG with no
std: IEG's raw write; with std: `std::print.useful.stdout["Hello, world!",
\n];`; and a limb of your own on top. The base has no Hello World: there
is nothing to write it in.

## Open

- The stable **plugin interface** for codextras and cores: compiler
  claude proposes it when building, the owner approves (2026-10-06).
- **The oracle and CCA**: nothing here is built; today both still have
  the operators built in and reach `std::` without a listing.
