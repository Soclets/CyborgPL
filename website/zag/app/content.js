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

// Home's examples (owner, 2026-10-15: "Next on Home, Examples", then "Replace two, except PhoneBook,
// with a low-level one, and the biggest one"): real programs from examples/, each shown with the
// output it really prints. The notes say only what the file itself shows.
export const HOME_EXAMPLES = [
  { id: "low_level", title: "Low level",
    note: "Raw memory from C through pointers, bits, a wrapping hash, SIMD, a C struct’s layout, and the OS through `write`, each unchecked step marked `no_check`." },
  { id: "corner_shop", title: "Corner shop",
    note: "The biggest example, {lines} lines: a day at a corner shop, from orders and receipts in exact decimals to the week’s report. Written to be read." },
  { id: "phonebook", title: "Phone book",
    note: "Maps, where a missing key is an answer, not a crash, and every argument after the first says its role: `key`, `value`." },
];
