# Good practice

How CyborgPL is meant to be written: the practices the language's rules
make easy, collected in one place. Proposed by design claude and approved
by the owner, 2026-10-02 ("all that you said there seems like a good
practice", then "write"). They are advice, not rules: the compiler
enforces none of them beyond what other files already decide.

**Names are free** (owner, 2026-10-02: "we don't need a guide on how to
name vars"). A quoted name may be anything, and no style for it is
recommended.

## Types and values

1. **Let Precog choose inside the program; write types at the doors.**
   `:=` everywhere inside; an explicit type where the outside world comes
   in — input, `extern`, struct fields, public functions. That is the
   third pillar used as meant (`core-features.md`).
2. **Narrow an outside value once, where it arrives, with a range type.**

   ```
   'text' := std::input.line[] otherwise "";
   'n' := to.int32[1, 99]['text'] otherwise 1;
   ```

   Everything downstream is then proven, with no guard at each use
   (`input.md`).
3. **No `mut` by habit.** Plain is the standard, and `change` already
   marks every change. Keep `immut` / `!mut` / `!$` for values whose
   fixedness matters to the reader: a rate, a limit.

## Safety

4. **The proven path first, then `check`.** A guard `if` in front of the
   site, and `check` on the ones that matter, so the compiler says whether
   the guard is enough. `nocheck` is for what no guard can prove, mostly C.
5. **Wrap `nocheck` once.** As std does: one small function holds the
   `nocheck` C call, and everything else calls that safe wrapper. The
   build's `nocheck` list stays short enough to review.
6. **`NO_FUCKING_CHECKS` only after measuring.** Never a first resort.
7. **`#strict` for public Limbs, plain files for programs.** A library's
   safety should not depend on who calls it, and `#strict` makes sure it
   does not.

## Ownership and failures

8. **`lend` by default, `give` when the receiver should own it, `copy`
   rarely.** Every `copy` is a visible allocation; many of them mean the
   design is off.
9. **Handle a failure where its cause is understood.** `otherwise` with
   one arm per reason close to the source, `otherwise fail` to pass it up.
   Careful with `otherwise 0` on input: it can turn "bad input" into a
   zero that looks real.
10. **`and.then` for guards.** `if 'i' </== list.length[lend 'xs'] and.then
    'xs'['i'] == 5 { … }`. A plain `and` runs both sides (`operators.md`).

## Loops and output

11. **A plain `for`; Precog parallelises.** `seq` only when order matters,
    `para` only after measuring, `simd` only when the plain loop did not
    vectorise, `!c` only for timing code.
12. **`print.useful` for people, `print` for debugging numbers,
    `print.format` for shaped output** (columns, hex, places).
