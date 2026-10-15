// Site copy. The core features are the owner's five pillars (design/core-features.md, decided
// 2026-09-14), in Zag's names (2026-10-14), with spellings the repo's examples use.
export const CORE = [
  { icon: "hourglass", title: "Everything at compile time",
    text: "Zag runs every possible thing at compile time to check for issues, so the final product is safe without paying for it when it runs. The developer waits; the final product doesn’t." },
  { icon: "key", title: "Ownership, no garbage collector",
    text: "Everything has exactly one owner, so the compiler knows when it can be freed. Every call says what happens to a value: `lend`, `give` or `copy`." },
  { icon: "type", title: "`:=` picks the best type",
    text: "Not just a type that fits: Zomp works out the best one from every value the binding will ever hold, across the whole program. Where a value comes from outside, you write its type." },
  { icon: "dots", title: "A dot narrows",
    text: "Most of the syntax is chained with dots, so you can flow with the code: `str.utf8`, `lend.mut`, `list.length`." },
  { icon: "chip", title: "Native code, through LLVM",
    text: "Zag compiles to native code through LLVM." },
];

// Home's examples (owner, 2026-10-15: "Next on Home, Examples"): real programs from examples/, each
// shown with the output it really prints. The notes say only what the file itself shows.
export const HOME_EXAMPLES = [
  { id: "fizzbuzz", title: "FizzBuzz", note: "The classic, in Zag’s words: `mod.floor`, `equal_to`, `at_most`, and `change` for every change." },
  { id: "bounded", title: "A bounded list", note: "A list that can never hold more than five. Going over is a failure, and the caller says what happens with `otherwise`." },
  { id: "phonebook", title: "A phone book", note: "Maps, where a missing key is an answer, not a crash, and every argument after the first says its role: `key`, `value`." },
];
