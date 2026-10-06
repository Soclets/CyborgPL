# Cores and limbs (owner, 2026-10-05; in progress, not built)

The owner: "I was thinking of making the base lang more lower-level ... I
wanted to move things like, plus, div, you know to std. And even more."
This note gathers what was decided that day. `DECISIONS.md` has each step
with the owner's words; this is the picture.

## The codextra (owner, 2026-10-06)

Above the cores in authority sits a **codextra**: the layer that holds the
language itself (types, `let`, `if`, loops, functions, the borrow rules,
likely Precog) and sets the rules cores play by. The base goes "as low
level as fuck" (owner): reading files, loading the codextra and cores and
enforcing their grants, the bridge to LLVM. When people say "CyborgPL",
they mean the official codextra, "like how they say BIOS for UEFI" (owner).
What exactly it holds is open.

## Two kinds of package

| | A **core** | A **limb** |
|---|---|---|
| What it is | "kinda more of like a compiler mod" (owner) | a library |
| Made of | a Rust plugin for CCA, plus CyborgPL code | CyborgPL |
| Extends the language (new syntax, checks, names without a prefix) | yes, through its plugin | no |
| Defines operators on the language's own types (`plus` on `int32`) | yes | only with that **power** granted |
| Gives Precog contracts it believes | yes, if verified | no: Precog proves a limb's code itself |
| Examples | **IEG**, the official core | **std**, your own libraries |

- **"Core" is a category**: anyone may write one, and cores may depend on
  cores (a GPU core can build on IEG). Cores are powerful enough to make
  dialects of CyborgPL, and that is fine (owner).
- **The official core, IEG** (the Intense Energy Generator), is raw and
  low-level, "not so practical for normal stuff" (owner). Its plugin adds
  `inline.LLVM { … }`, LLVM IR written in place: portable assembly that
  LLVM still optimises, unlike `inline.S`. It is verified: RRx2 checks its
  IR against its contracts.
- **An unverified core is used with a warning, never refused** ("Maybe
  it's their own, maybe it's their friend's", owner): its contracts are
  believed only as far as its author is trusted. Installing a core is
  already trusting it, as installing any program is.
- **A core need not put its name in the syntax** (no `ieg::` unless it
  wants one).

## The layers

```
the base language   syntax, types, borrows, the "exact, checked where it lands" rules
IEG (core)          raw machine operations; inline.LLVM
other cores         anything, even dialects; may build on IEG
a parallel core     splits proven loops across cores (threads); needs IEG;
                    a kernel leaves it out and runs its loops in order
a GPU core          GPU offload and inline.MSL; needs IEG
std (limb + power)  plus, minus, times, div.floor …, pow, printing,
                    friendly names for LLVM operations (std::bits.count_ones …)
your limbs          everything else
```

std is written in CyborgPL (2026-09-30), and may call the `no_std` Rust
runtime for whatever CyborgPL cannot express yet; the Rust parts shrink
over time. Its operators are ordinary CyborgPL on top of IEG's raw
operations, so Precog still checks them and LLVM still sees one `add`.

## Powers

A limb may be granted powers, "just what's necessary" (owner), so one for
now: **defining operators on the language's own types**, which std needs.
A power is declared in the limb's manifest and shown by every build that
uses it:

```
# CCA.cplr
const 'powers' := [cca::Power.operators_on_builtin_types,];
```

## Manifests

A limb says which cores it is built for, in a list of its own beside its
dependencies, so a file in a core's dialect is never a surprise:

```
# CCA.cplr
const 'cores' := [cca::Core[name = "IEG"],];
const 'dependencies' := [cca::Dep[path = "../std"],];
```

`Synthia.ttsr` has the same as a `[my cores]` section. Spellings are design
claude's, not yet ruled.

## Installing

The download from the website is **CCA and IEG, nothing more**; std is
installed like any limb ("the highest shit you can get from the website
is, by installing the language is a core", owner). `cca new` writes an
empty `CCA.cplr` ("I don't think you're a beginner, if you sign up for
Precog", owner). CCA says which IEG versions it works with and refuses a
mismatch.

## Hello World at every level

The base language through `inline.S` (raw system calls), through C
(`extern.C fn write`), inside IEG (`inline.LLVM`, which only a core may
write), through std (`std::print.useful.stdout["Hello, world!", \n];`),
and through a limb of your own on top. Each step down trades convenience
for control; the `nocheck`s show where safety becomes the writer's.

## Where the speed lives (owner, 2026-10-05)

Precog's proofs and SIMD are the base language's: they need nothing at run
time. Running a loop across cores needs threads, so it comes from a
higher-level official core that requires IEG; a program without it (an OS
kernel) runs every loop in order, still proven and still SIMD, and `for
para` is refused there. GPU offload is a core of its own, also requiring
IEG, and `inline.MSL` belongs to it (owner, 2026-10-06: "`b`"). Both
names are open.

## Open

- **What the base keeps**: whether `==`, `<`, `and`, `or`, `not` and `++`
  move to std as `plus` does, or stay in the language.
- **How a core is installed and named** (beside CCA; its version match).
- **The plugin interface** CCA offers cores: it must be stable, so CCA's
  insides can change without breaking every core (design claude's note).
- **The unverified-core warning's wording.**
- **The oracle**: it keeps its built-in meaning for IEG's and std's
  operators; RRx2 checks IEG's IR. Nothing here is built yet, and today's
  oracle and CCA still reach `std::` without a listing.
- **A naming clash, noted by design claude**: "core" also means a CPU core
  in these notes (`parallel on N cores`, `CLAUDE.md`'s core split).
