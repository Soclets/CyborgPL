# Good practice

> Revised 2026-10-07: code here is in the new syntax; an old spelling in the prose is history. The old → new table is `revision-2026-10-07.md`.

How Nerve is meant to be written: the practices the language's rules
make easy, collected in one place. Proposed by design claude and approved
by the owner, 2026-10-02 ("all that you said there seems like a good
practice", then "write"). They are advice, not rules: the compiler
enforces none of them beyond what other files already decide.

**Names are free** (owner, 2026-10-02: "we don't need a guide on how to
name vars"). A quoted name may be anything, and no style for it is
recommended.

## Types and values

1. **Let Cortex choose inside the program; write types at the doors.**
   `:=` everywhere inside; an explicit type where the outside world comes
   in — input, `extern`, struct fields, public functions. That is the
   third pillar used as meant (`core-features.md`).
2. **Narrow an outside value once, where it arrives, with a range type.**

   ```
   let 'text' := cord::input.line[] otherwise "";
   let 'n' := to.int32[min 1, max 99]['text'] otherwise 1;
   ```

   Everything downstream is then proven, with no guard at each use
   (`input.md`).
3. **No `mut` by habit.** Plain is the standard, and `change` already
   marks every change. `mut` is a checked promise that a value changes
   (2026-10-10: one that nothing changes is refused), so write it where the
   change is the point. Keep `immut` / `no_shadow` for values whose
   fixedness matters to the reader: a rate, a limit.

## Safety

4. **The proven path first, then `check`.** A guard `if` in front of the
   site, and `check` on the ones that matter, so the compiler says whether
   the guard is enough. `no_check` is for what no guard can prove, mostly C.
5. **Wrap `no_check` once.** As `cord` does: one small function holds the
   `no_check` C call, and everything else calls that safe wrapper. The
   build's `no_check` list stays short enough to review.
6. **`NO_FUCKING_CHECKS` only after measuring.** Never a first resort.
7. **`#strict` for public Neurons, plain files for programs.** A library's
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
10. **`and.then` for guards.** `if 'i' at_most list.length[lend 'xs'] and.then
    'xs'[pos 'i'] equal_to 5 { … }`. A plain `and` runs both sides (`operators.md`).

## Loops and output

11. **A plain `for`; Cortex parallelises.** `seq` only when order matters,
    `para` only after measuring, `simd` only when the plain loop did not
    vectorise, `no_cheat` only for timing code.
12. **`print.useful` for people, `print.stored` for debugging numbers,
    `print.format` for shaped output** (columns, hex, places).
13. **`precedence.low_first` for a Neuron's operator levels** (owner,
    2026-10-03: "low_first is good practice"): level 1 binds loosest, as a
    precedence table is read top to bottom. `precedence.high_first` is
    there for whoever thinks the other way.
