// Site copy. Claims come from design/core-features.md and design claude (2026-09-23);
// the description is design claude's, reviewed by the owner on the page.
export const REPO_URL = "https://github.com/Cyborg-Assembly-Manufacturing-Site/CyborgPL";

export const DESCRIPTION =
  "CyborgPL is a compiled language where Precog, a whole-program analyser, proves every overflow and bounds check at compile time. The final build carries none of them, or it doesn’t build.";

export const FEATURES = [
  { id: "precog", icon: "shield", title: "Precog", text: "Every overflow and bounds check is proven at compile time — or the program doesn’t build." },
  { id: "ownership", icon: "key", title: "Ownership, no GC", text: "Every value has one owner. give, copy and lend say what happens to it." },
  { id: "inference", icon: "type", title: ":= picks the best type", text: "Worked out from the whole program. A value from outside needs its type written." },
  { id: "dot", icon: "dot", title: "A dot narrows", text: "str.utf8, lend.mut, list.length, map.ordered — one small rule, everywhere." },
  { id: "failures", icon: "alert", title: "Failures in signatures", text: "-> T or fail[name], handled with otherwise." },
  { id: "llvm", icon: "chip", title: "LLVM backend", text: "Native code, through LLVM." },
];

export const ACCENTS = ["#6c71c4", "#268bd2", "#2aa198", "#859900", "#cb4b16", "#d33682"];

// What trips up newcomers, per design claude. Every code line is from design claude
// (DECISIONS.md) or copied from a real program in examples/ — none are invented.
// Syntax as of 2026-09-27: no `let`, `change`, no `pos`.
export const SYNTAX = [
  { title: "Names are quoted", text: "Variables always sit in single quotes. Types, functions and fields are bare words.", code: "'x' := 1000;" },
  { title: ":= or =", text: "A declaration starts with its name. := works out the best type from the whole program; = needs the type written.", code: "'x' := 1000;\n'n': int32 = 5;" },
  { title: "Changing says change", text: "Every change to a value says change, so it never looks like a declaration. Only a mut value can change.", code: "mut 'x' := 1000;\nchange 'x' = 2000;" },
  { title: "Calls use brackets", text: "Arguments go in [ ], not ( ).", code: "add[1, 2]" },
  { title: "x multiplies", text: "x multiplies, xx is power, mod is the remainder.", code: "'by3' := ('n' mod 3) == 0;" },
  { title: "Positions start at 1", text: "The first element is position 1, written bare.", code: "return 'xs'[1];" },
  { title: "The newline is outside the string", text: "A string is exactly its characters. The newline is its own piece.", code: "std::print.stdout[\"Hello, World!\", \\n];" },
  { title: "Ownership at the call site", text: "The caller says whether it lends, gives or copies.", code: "describe[lend 'l']" },
  { title: "A dot narrows", text: "The thing on the left, made more specific: str.utf8, lend.mut, list.length.", code: "if list.length[lend 'xs'] == 0 { fail empty; }" },
  { title: "Comments", text: "# runs to the end of the line.", code: "# like this" },
];

export const LICENCE = [
  { title: "Code", text: "The compiler, the tools, the libraries and this website’s code: Apache-2.0 with the LLVM exception. Using CyborgPL owes nothing — compiled programs carry no notice." },
  { title: "This site’s text", text: "CC BY 4.0: copy, translate and reuse it, with credit." },
  { title: "Names and logo", text: "CyborgPL, Precog, Limb and Synthia, and the logo, are trademarks. They aren’t licensed with the code: a fork can’t call itself CyborgPL." },
];
