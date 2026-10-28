# Good practice

> Revised 2026-10-07: code here is in the new syntax; an old spelling in the prose is history. The old → new table is `revision-2026-10-07.md`.

How CyborgPL is meant to be written: the practices the language's rules
make easy, collected in one place. Proposed by design claude and approved
by the owner, 2026-10-02 ("all that you said there seems like a good
practice", then "write"). They are advice, not rules: the compiler
enforces none of them beyond what other files already decide.

**Names are free** (owner, 2026-10-02: "we don't need a guide on how to
name vars"). A quoted name may be anything, and no style for it is
recommended.

## Types and values

1. **Let the prover choose inside the program; write types at the doors.**
   `:=` everywhere inside; an explicit type where the outside world comes
   in — input, `extern`, struct fields, public functions. That is the
   third pillar used as meant (`core-features.md`).
2. **Narrow an outside value once, where it arrives, with a range type.**

   ```
   let $text := std::input.line() ?? "";
   let $n := to.i32[min: 1, max: 99](lend $text) ?? 1;
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
5. **Wrap `no_check` once.** As `std` does: one small function holds the
   `no_check` C call, and everything else calls that safe wrapper. The
   build's `no_check` list stays short enough to review.
6. **`NO_FUCKING_CHECKS` only after measuring.** Never a first resort.
7. **A public kit states its limits in its signatures** (owner,
   2026-10-12: "(b), if it's actually valid"; `#strict` did this until
   2026-10-07). A library's safety should not depend on who calls it, and
   the prover proves a site partly from its callers. So range types on public
   parameters (`'p': i32[min: 0, max: 100]`): the function is proven from
   its signature, and a bad value is refused at the caller's line. And
   lifetime groups (`@x`) on returned borrows even where they could be
   inferred, so a reader sees what the result depends on.

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

11. **A plain `for`; the prover parallelises.** `seq` only when order matters,
    `para` only after measuring, `simd` only when the plain loop did not
    vectorise, `no_cheat` only for timing code.
12. **`print.useful` for people, `print.stored` for debugging numbers,
    `print.format` for shaped output** (columns, hex, places).
13. **`precedence.low_first` for a kit's operator levels** (owner,
    2026-10-03: "low_first is good practice"): level 1 binds loosest, as a
    precedence table is read top to bottom. `precedence.high_first` is
    there for whoever thinks the other way.

## Collections

14. **A preset before a template, a template before copies** (owner,
    2026-10-13: "yea"). A narrowing that only fixes arguments is a
    preset, `quick ['step' = 2];`. A template is for members a preset
    cannot make, where the value changes the signature (`zero.of_int16`,
    `zero.of_int64`) or no main function takes it. Members that differ in
    what they do are written out.

## Words and symbols

15. **Use the symbols for the plain form; write the word for a variation** (owner, 2026-10-16: "make
    it good practice to use symbols, and when you want another variation, you use the word form").
    `'a' + 'b'`, `'a' x 'b'`, `'b' xx 10`, `'i' < 10`, `-'x'`; but `'a' plus.wrap 'b'`, `'a' div.floor
    'b'`, `negate.ieee 'x'`. Code with words "takes long to skim", so a symbol is kept only where it reads
    correctly (`+ - x xx < >`, a leading `-`); everything else has no symbol and is always the word:
    `equal_to`, `not_equal`, `at_most`, `at_least`, `div.…`, `mod.…`, `and`, `and.then`, `or.else`,
    `concat`, `lend`, `ptr[T]`, `exactly`. A change is written out, `change 'i' = 'i' + 1;`. (2026-10-14
    to 16 this item said the opposite: "Write the words, not the symbols".)

18. *(Reversed 2026-10-26: "use `%` behind" ended when `%` became the limb mark; a percentage is `80 percent`, its
    type `percent[d32]`.)*

## Names

17. **Good naming depends on what you would use** (owner, 2026-10-16). A quoted name may hold any printable
    ASCII character (owner, 2026-10-16), a `'` written `\'` and spaces one at a time, so `'80%'`, `'v1.2'`,
    `'total due'` and `'it\'s'` are all names; pick the one that says what the code using it means.

19. *(Withdrawn 2026-10-22: "write std's names bare" ended with the `bare.` marks; another kit's names, std's
    included, always take its path.)*

20. *(Withdrawn 2026-10-22: "mark your own names `optional`"; there are no marks.)*

## Shape of a program

16. **Keep `MAIN` thin** (owner, 2026-10-15: "yea"). `MAIN` reads the input, calls the real work, and
    reports the result; the work lives in modules, one topic per file, so `kit.cy` reads like a table of
    contents. A reader sees the program's shape at once, and small focused functions are what the prover
    proves best. As kayo and the oracle do: their `main.rs` hands everything to the parser, checker, prover
    and code generator.
