// Site copy. Claims come from design/core-features.md and design claude (2026-09-23);
// the description is design claude's, reviewed by the owner on the page.
export const REPO_URL = "https://github.com/Soclets/Nerve-lang";

// RRx2, Nerve's own verifiers (owner, 2026-10-01: link it). Its repo for now — rrx2.com, which
// will be its site, doesn't resolve yet; switch to it once it does.
export const RRX2_URL = "https://github.com/Cyborg-Assembly-Manufacturing-Site/RRx2";

export const DESCRIPTION =
  "Nerve is a compiled language where Precog, a whole-program analyser, proves every overflow and bounds check at compile time. The final build carries none of them, or it doesn’t build.";

export const FEATURES = [
  { id: "precog", icon: "shield", title: "Precog", text: "Every overflow and bounds check is proven at compile time — or the program doesn’t build." },
  { id: "ownership", icon: "key", title: "Ownership, no GC", text: "Every value has one owner. `give`, `copy` and `lend` say what happens to it." },
  { id: "inference", icon: "type", title: "`:=` picks the best type", text: "Worked out from the whole program. A value from outside needs its type written." },
  { id: "dot", icon: "dot", title: "A dot narrows", text: "`str.utf8`, `lend.mut`, `list.length`, `map.ordered` — one small rule, everywhere." },
  { id: "failures", icon: "alert", title: "Failures in signatures", text: "`-> T or fail[name]`, handled with `otherwise`." },
  { id: "llvm", icon: "chip", title: "LLVM backend", text: "Native code, through LLVM." },
];

// Home's honest table (owner, 2026-09-28): each feature beside what it costs. Every line is
// the design's own words, from design/core-features.md and DECISIONS.md — no costs invented.
export const TRADEOFFS = [
  { get: "Precog proves every overflow and bounds check at compile time; the final build carries none of them.",
    cost: "Compile time is not a cost the language cares about: builds are slow, and every build is a full one — there is no fast development mode." },
  { get: "The working build checks what Precog can’t yet prove, whatever the runtime cost.",
    cost: "The final build refuses that code until you write a guardrail, a tighter type, or `no_check`." },
  { get: "Ownership, no garbage collector: every value has one owner, so the compiler knows when to free it.",
    cost: "Every call says what happens to a value: `lend`, `give` or `copy`." },
  { get: "`:=` works out the best type from the whole program.",
    cost: "A value from outside the program has no bound, so there you write its type; `:=` on it is an error." },
  { get: "Failures are part of a function’s signature, not exceptions.",
    cost: "The caller must handle every one; forgetting is a compile error." },
  { get: "`and` / `or` always work out both sides: nothing written is skipped.",
    cost: "A check that guards the next step takes two `if`s, or `and.then`." },
  { get: "An old spelling is refused with an error that names the new one.",
    cost: "No stability promise, ever: Nerve is unstable, and stays that way." },
  { get: "Native code, through LLVM.",
    cost: "In development: no release yet, nothing to download." },
];

export const ACCENTS = ["#6c71c4", "#268bd2", "#2aa198", "#859900", "#cb4b16", "#d33682"];

// Introductory syntax, kept in step with the programs in examples/.
export const SYNTAX = [
  { title: "Names are quoted", text: "Variables always sit in single quotes. Types, functions and fields are bare words.", code: "let 'x' := 1000;" },
  { title: "`let` binds a value", text: "Use `let` to introduce a value. `:=` infers its type; with `=`, write the type explicitly.", code: "let 'x' := 1000;\nlet 'n': int32 = 5;" },
  { title: "Changing says `change`", text: "Declare a value `mut` before changing it. `change` marks an assignment.", code: "let mut 'x' := 1000;\nchange 'x' = 2000;" },
  { title: "Calls use brackets", text: "Arguments go in `[ ]`, not `( )`.", code: "add[1, 2]" },
  { title: "Arithmetic uses words", text: "`plus`, `minus` and `times` are operators. Division and remainder name their rounding, such as `div.floor` and `mod.floor`; `pow` raises to a power.", code: "let 'by3' := ('n' mod.floor 3) == 0;" },
  { title: "Positions start at 1", text: "The first element is position 1, and every position says `pos`.", code: "return 'xs'[pos 1];" },
  { title: "The newline is outside the string", text: "A string is exactly its characters. The newline is its own piece, `newln`.", code: "cord::print.stored[\"Hello, World!\", newln];" },
  { title: "Ownership at the call site", text: "The caller says whether it lends, gives or copies.", code: "describe[lend 'l']" },
  { title: "A dot narrows", text: "The thing on the left, made more specific: `str.utf8`, `lend.mut`, `list.length`.", code: "if list.length[lend 'xs'] equal_to 0 { fail empty; }" },
  { title: "Comments", text: "`#` runs to the end of the line.", code: "# like this" },
  { title: "Comparison uses words", text: "`equal_to`, `not_equal`, `at_most` and `at_least`; only `<` and `>` stay symbols.", code: "while 'i' at_most 'limit' {" },
];

// Each licence links to its official text, and to the exact file in the repo (the repo's
// LICENSE is Apache-2.0 with the LLVM exception appended; LICENSE-CC-BY-4.0 is the site's text).
export const LICENCE = [
  { title: "Code", text: "The compiler, the tools, the libraries and this website’s code: Apache-2.0 with the LLVM exception. Using Nerve owes nothing — compiled programs carry no notice.",
    links: [
      { label: "Apache License 2.0", href: "https://www.apache.org/licenses/LICENSE-2.0" },
      { label: "LLVM exception", href: "https://spdx.org/licenses/LLVM-exception.html" },
      { label: "LICENSE in the repo", href: `${REPO_URL}/blob/main/LICENSE` },
    ] },
  { title: "This site’s text", text: "CC BY 4.0: copy, translate and reuse it, with credit.",
    links: [
      { label: "CC BY 4.0", href: "https://creativecommons.org/licenses/by/4.0/" },
      { label: "Legal code", href: "https://creativecommons.org/licenses/by/4.0/legalcode" },
      { label: "LICENSE-CC-BY-4.0 in the repo", href: `${REPO_URL}/blob/main/LICENSE-CC-BY-4.0` },
    ] },
  { title: "Names and logo", text: "Nerve, Precog, Neuron and Synthia, and the logo, are trademarks. They aren’t licensed with the code: a fork can’t call itself Nerve." },
];
