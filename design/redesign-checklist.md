# Redesign checklist

> Owner, 2026-10-28: "Please make a big redesign be considered more throughly", then "yes, record it". Each step is one
> that would have caught, before the build, something found after redesign three was built.

A redesign is **big** when it changes a symbol, a keyword or a block's form, or touches more than about 10 examples.
Smaller rulings skip the list.

Before the owner rules on a big redesign, design claude does these steps and shows what each found:

1. **Confusion check.** For every symbol and word that is new or changes meaning, what it means in C, C++, Java,
   JavaScript, Python, Rust, Go, C# and Swift. Flag any that would run with another meaning rather than be refused.
   (Missed: `^`, power here and XOR there, found only after the build.)
2. **Frequency count.** How often the thing is used in real code before choosing a symbol, a word or std for it:
   Rust crates under `~/.cargo/registry`, Python's standard library, `bench/`. Counted with a tool, never guessed.
   (How `std::pow` was settled: 40 powers against 133 XORs in 1,270,622 lines of Rust.)
3. **Rule check.** Each change against the naming rule and its symbol clause (`syntax-sketch.md`): a bare name is the
   obvious default, a narrowing a word, a symbol only where maths or nearly every language gives it exactly that meaning;
   and against what is the base's and what is std's. (Missed: `++`, and `std::plus` still accepted.)
4. **Twins.** Each change's counterpart, decided together: `->` / `<-`, `+` / `+=`, `print` / `println`, a keyword and
   its closer.
5. **Whole programs.** `night_trains.cy` and two other examples converted and read whole before the ruling, not only a
   table of spellings.
6. **Outside read.** When multifinity offers one, their Claude reads the converted programs. It is their work, not a step
   design claude or the owner runs (owner: "the fresh Claude stuff is mulf's work, I don't know how to do those").
7. **Build.** Only on the owner's go, and done only when `cargo test`, `oracle/tools/stale_prose.py` and the round trip
   (`oracle/tools/redesign_round_trip.py` on the pre-redesign examples) are all clean, or their failures are listed.
