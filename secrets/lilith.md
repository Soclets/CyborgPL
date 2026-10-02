# Lilith

An idea of the owner's, 2026-10-02, written while waiting on RRx2 tests.
Not part of CyborgPL, and not planned. (Not a secret either, in both
meanings of the folder's name: no passwords here.)

## The idea (owner's words)

> `Lilith` (`.lth`), JIT (`.lthJ`).
> Lilith is a JIT computer language that uses a from-scratch compiler
> backend, without fully reverse engineering its targets — by looking at
> LLVM's code. It also has the ability to use other languages inline
> (e.g. Assembly), by trading portability for low-level control.

## Q & A (owner's)

**Q.1:** "Are you abandoning CyborgPL."
**A.1:** "No, Lilith is just an idea."

**Q.2:** "So, when are you gonna create Lilith?"
**A.2:** "Maybe never"

**Q.3:** "Where did you get the name?"
**A.3:** "From a hentai anime 🤦"

## A note from design claude

LLVM's target descriptions (the TableGen `.td` files: each processor's
instruction encodings, registers and quirks) are Apache-2.0 WITH
LLVM-exception, the owner's own licence, so learning from them is fine. A
JIT built on those facts but not on LLVM itself would start much faster
than an LLVM-based JIT, which is where LLVM is weakest for JITs.
