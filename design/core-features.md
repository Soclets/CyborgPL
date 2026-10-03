# Core features of CyborgPL

> The compile-time technology described under "Everything at compile time"
> is named **Precog** (decided 2026-09-14).

Decided by the owner, 2026-09-14. These are the five pillars every later
design decision is measured against.

## Everything at compile time

CyborgPL runs every possible thing at compile time to check for issues,
ensuring final-product safety without sacrificing runtime performance.

Compile time is not a cost the language cares about. The developer waits;
the final product does not.

## Safety via the ownership model

Everything has exactly one owner, so the compiler knows when it can be freed —
safety without sacrificing runtime performance.

## `:=` isn't just lazy inference

The compiler works out the **best** type for a `:=` binding at compile time,
not merely *a* type that fits.

## Dot-chained syntax

Most of the syntax uses dot chaining, so a reader can "flow with the code".

## LLVM as the backend

CyborgPL compiles to native code through LLVM.

### What "best" means (decided 2026-09-14)

Best for **safety and performance** of the final product. The compiler
analyses every value the binding will ever hold, across the whole program,
and picks the type that is provably safe and fastest for that set of values.
It can afford to do this because compile time is unlimited.

"Best" is *not* "smallest". A lone `int8` scalar is no faster than an
`int32` on a 32/64-bit register machine and sometimes slower; for scalars
the best width is usually the machine's natural one, while for lists and
structs — where memory bandwidth and cache lines dominate — narrower is
genuinely better. The larger runtime win is not the width at all but the
*proof* that came with it: a proven range removes overflow and bounds checks
and is handed to LLVM as `nsw`/`nuw`/`!range`, unlocking optimisations that
Go's and Rust's local inference can never justify.

When no bound can be proven — the value comes from outside the program — `:=`
is a compile error. The programmer writes an explicit type at that boundary,
and everything downstream is provable again. The only place a human writes a
type is where the program touches the outside world.

### Unproven safety properties (decided 2026-09-14)

The compiler runs the whole program at compile time over *sets* of values:
known values are executed outright, bounded values (explicit types at the
boundary) are abstractly interpreted. Every safety property — overflow, bounds,
division by zero — is a question asked of those sets.

Where a property cannot be proven, the compiler **inserts a runtime check, no
matter the performance cost**. That check is removed only when the programmer
has written a real guardrail (validation) and explicitly disables that one
check via syntax. The final product's safety comes from programmer guardrails
the compiler can see, not from compiler checks left behind.

### No development mode (decided 2026-09-16)

There is no build that skips Precog, no "prove later" tier, no fast path for
everyday edits. Every build is a full build; the tagline is the contract.
Precog gets faster by engineering — including not re-proving functions
whose bodies and inputs are byte-identical to the last build, which reuses
proofs rather than skipping them — never by doing less.

### The three check tiers (decided 2026-09-14; spelling decided 2026-09-19)

- **default** — "I have no guardrails yet, and I want you to help me." In
  the **working build** the compiler inserts a runtime check wherever it
  cannot prove safety, so the program runs and fails loudly where a
  guardrail is missing. In the **final build** (owner, 2026-09-20: the
  runtime-check rule "was when building around, dev testing, not the
  final product") the same site is a **compile error with a
  counterexample** — the default behaves as `check`. Both builds run
  Precog in full; the final build is stricter, never looser, so this is
  not the rejected development mode. The final product contains no
  runtime checks: every site is proven, `check`-confirmed, or `nocheck`.
  Proposed: `synthia build` / `synthia final`, Synthia's, so no file-wide
  switch enters the language.
- **`check`** — "I've built a guardrail, could you help me verify it?" The
  compiler proves the guardrail exactly; proven → check removed, otherwise a
  compile error **with a counterexample** — the value that gets past the
  guardrail.
- **`nocheck`** — "Okay, I'm sure this is correct." Check removed, no
  verification. The one place the final product's safety rests on the
  programmer alone. The build output lists every `nocheck` site.
  **A certain failure gets a note, never a refusal** (owner,
  2026-09-22): when Precog's ordinary work happens to show that a
  `nocheck` site fails on *every* run — `nocheck 'q' := 1 div.floor 'z';`
  with `'z'` proven to be 0 — the build says so (`note:`, the third
  kind in `diagnostics.md`) and builds anyway. Precog never goes
  looking; it only refuses to stay silent about what it could not
  avoid noticing. A `nocheck` that fails on *some* inputs ships with
  nothing said, so "no verification" holds everywhere it matters.
  Proposed alternatives: say nothing at all; refuse the certain case
  (which would make `nocheck` not quite mean what it says).

**Spelling** (owner, 2026-09-19: "`check` & `nocheck` at the front"): a bare
word in front of a statement, covering every check site inside it. A `{ }`
block is a statement, so a region is `check { … }` with no second rule.

```
check change 'xs'['i'] = 'total' plus 'score';     # prove: index in range, add fits
nocheck change 'hash' = 'hash' times 31 plus 'c';          # I want the wrap; on my word
check { for 'i' in range[1, 'n'] { … } }     # every site in the region
check if 'x' > 0 { … }                       # an if or a loop is a statement too
```

- Innermost word wins when they nest. **No alias** for `nocheck` (owner,
  2026-09-19): it stays long and rare; a dense region uses `nocheck { … }`.
- Never on an expression (`'a' plus (check 'b')`): if half a statement needs
  it, split the statement. Never on a `fn`, `struct`, `const`, or `module`:
  a function's safety is the sum of its statements, and a `check fn` would
  be a global by another name. Never at the top level: **the tiers are never
  global** — no flag, no file-wide switch, no release mode (see "No
  development mode"). The grammar has nowhere to write a global one.
- **`NO_FUCKING_CHECKS`, the fourth tier** (owner, 2026-09-27): everything
  `nocheck` turns off, and what C hands back is trusted too, so a C enum or
  range type from C gets no `out_of_range` check and needs no `otherwise`
  for one. It goes where `nocheck` goes, in front of a statement or a
  block, never on a function or a file, and it is listed in the build
  output with the `nocheck` sites. For the rare hot spot where even a
  comparison per value from C matters, and the author accepts undefined
  behaviour if C sends a number the binding doesn't know. It does not
  touch failures of Cyborg's own operations (text that isn't a number, a
  missing map key); those are answers, not checks. That limit is design
  claude's reading of "no checks", not asked. The owner chose the name.
  Proposed first as `nocheck.all`.
- **`#strict`: every reason written where it is used** (owner, 2026-09-27:
  a). In a file marked `#strict`, a check site must be proven from what
  is written in its own function: a guardrail `if`, a range type (on a
  parameter, a variable, a declared return), or a loop's bounds. Precog
  still checks the proof, but may not use facts from callers or other
  functions' bodies, so a reader sees why each site is safe, and a
  function stays safe whoever calls it later. "Verbose, but for a
  reason." A comment needs a space (`# text`), and `#strict` is one of the
  `#word` forms, like `#wl`. It is a file-wide switch, which the
  "tiers are never global" rule above forbade in its wording; it only
  makes things stricter, never looser, and the owner chose it. Proposed
  alternative: an `if` in front of every site even where a type already
  makes it safe.

  **`nocheck` is allowed** in a strict file (owner: a): it is a written
  reason too, and listed. Otherwise a C call could never sit in one.

  **And the file writes out what others leave to Precog** (owner, same
  day: "`#strict` gives a fuck about the reader too"):
  - `pos` on every position: `'xs'[pos 2]`, `list.slice[lend 'xs', pos 2,
    pos 4]`, `simd.load[lend 'xs', pos 'i']`. Only strict files have it;
    elsewhere `pos` is gone. So a list read never looks like a call.
  - a type on every parameter;
  - a return type on every function that returns a value;
  - `mut` or `immut` (or `!mut`) on every declaration;
  - `temp` or `perm` on every `for`;
  - a `match` / `otherwise` case says its value with `giveS` (2026-09-29,
    `enums.md`);
  - a returned borrow names its lifetime group, `--> lent@x T` (2026-09-29,
    `ownership-syntax.md`);
  - each range bound labelled, the step always written:
    `for temp 'i' in range[from 2, to 8, by 2]` (owner, same day: "the loop
    syntax should be more self-explanatory"). "Count from 2 to 8 by -4"
    shows at a glance that it never runs. Proposed alternative words:
    `start` / `end` / `step`. Outside strict files the labels are not
    used, as with `pos`.
  - the count of a `list.fill` labelled: `list.fill[0, count 10]`
    (2026-09-30, `lists.md`).
  Proposed and not chosen: a type on every variable, since `:=` letting
  Precog pick the best type is a pillar and a written type can be a worse
  choice. Also proposed for mutability: `mut` only on variables that
  change (4a), or nothing, since `change` already marks each change (4c).
  The owner chose `mut` or `immut` everywhere (4b).

  `#strict` goes once, at the top of the file, before any item, and covers
  the file and its `module` blocks. That is the oracle's reading and was
  not asked. **The working build refuses too** (owner: a): in a strict
  file a site not proven from its own function is an error in every
  build, not a run-time check until the final one. The reason must be
  written, so you find out at once. Proposed alternative: the working
  build checks at run time, as elsewhere.
- **The ladder** (clarified 2026-09-19): a guardrail that is good enough
  already removes the check under the default tier — Precog proves it and
  emits nothing; a proven check can never fire, so keeping it would be pure
  cost. `check` does not remove anything extra; it makes the compiler *tell
  you* when the guardrail is not enough. So the finish line is a passed
  `check`, and `nocheck` is the exit for the site no guardrail can make
  provable — never the last step after building one.
- Build output lists every `nocheck` site and every passed `check` (owner,
  2026-09-19). Proposed, unanswered: also the default-tier sites still
  carrying a runtime check — the "where to add a guardrail" list.
- A guardrail is an ordinary `if` in front of the site; `check` asks Precog
  to confirm the `if` is enough:

  ```
  if 'x' > 65_504 { deny[]; } else {
      check 'y': bin16 = 'x';     # proven from the branch: check removed
  }
  ```
  Without the `if`, the same line is a compile error: *"'x' does not fit
  bin16 when input is 70000."*
- The oracle cannot prove, so it runs every check and reports what the
  compiler would have found: a `check` whose site fires names the missing
  guardrail with the concrete value; a `nocheck` whose site fires "lied".

### Ownership with borrowing (decided 2026-09-14)

One owner per value; freed when the owner's scope ends. Moves transfer
ownership, copies make a new value, and borrows give temporary access with the
compiler proving the borrow never outlives its owner and never aliases a
mutation.

Lifetimes are inferred by default — the whole-program analyser sees every
call site. An optional lifetime annotation pins a function's borrow contract,
which the compiler then verifies. There is no unverified form.
