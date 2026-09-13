# Core features of CyborgPL

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

### The three check tiers (decided 2026-09-14; keyword spellings placeholder)

- **default** — "I have no guardrails yet, and I want you to help me." The
  compiler inserts a runtime check wherever it cannot prove safety.
- **`check`** — "I've built a guardrail, could you help me verify it?" The
  compiler proves the guardrail exactly; proven → check removed, otherwise a
  compile error saying what fact is missing.
- **`nocheck`** — "Okay, I'm sure this is correct." Check removed, no
  verification. The one place the final product's safety rests on the
  programmer alone.
