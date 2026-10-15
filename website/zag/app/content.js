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

// Home's examples (owner, 2026-10-15: "Next on Home, Examples", then "Hello World (default selected),
// C low-level + incl, and ASM low-level"): real programs from examples/, each shown with the output
// it really prints. The notes say only what the file itself shows. The first is open at the start.
export const HOME_EXAMPLES = [
  { id: "hello", title: "Hello, World",
    note: "The whole program: a `MAIN`, and one print from `std`." },
  { id: "c_headers", title: "C, through incl",
    note: "C’s own headers, read while building: `incl.C.system` brings in `write`, `strlen` and the `#define` constants, with no hand-written `extern.C`. Every call into C is a `no_check` site." },
  { id: "asm_beside_zag", title: "Assembly",
    note: "ARM64 assembly inline, beside the same work in plain Zag: `madd` multiplies and adds in one instruction, `clz` counts leading zeros. `with` binds variables to registers, and assembly is always a `no_check` site." },
];
