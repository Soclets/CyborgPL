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
