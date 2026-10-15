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

// Pros and cons (owner, 2026-09-28, for Nerve's site: an honest table; 2026-10-15, for Zag's: "I
// feel like the 'What it costs' isn't really accurate", "just make it not supposed to be read
// side-by-side as tradeoffs", then "just fucking say Pros and Cons"). Two lists, each line standing
// on its own. Every line is the design's own words, from design/core-features.md, design/failures.md
// and DECISIONS.md, in Zag's names; nothing is invented.
export const PROS = [
  "Zomp proves every overflow and bounds check at compile time, and the final build carries none of them.",
  "The working build checks what Zomp can’t prove yet, whatever that costs when it runs.",
  "Ownership and no garbage collector: every value has one owner, so the compiler knows when to free it.",
  "`:=` works out the best type from the whole program.",
  "Failures are part of a function’s signature, not exceptions.",
  "`and` and `or` always work out both sides: nothing written is skipped.",
  "An old spelling is refused with an error that names the new one.",
  "Native code, through LLVM.",
];
export const CONS = [
  "Builds are slow, and every build is a full one. There is no fast development mode.",
  "Where Zomp can’t prove something, the final build refuses the code until you write a guardrail, a tighter type, or `no_check`.",
  "Every value you pass says what happens to it: `lend`, `give` or `copy`.",
  "A value from outside the program needs its type written; `:=` on it is an error.",
  "Every failure a function can return must be handled; forgetting one is a compile error.",
  "When one check guards the next, it takes two `if`s, or `and.then`.",
  "Zag is being experimented with, so please don’t expect stability: any release may change or remove anything.",
];

// What Zag's own tools are made with (owner, 2026-10-12 for Nerve's Home: "add host langs", and "we'll
// eventually have to use C++ too"; 2026-10-15 for Zag's: "Yeah"), as zomp/README.md, each crate's
// Cargo.toml and design/implementation-languages.md say.
export const BUILT_WITH = [
  { part: "Zomp, the compiler, and its prover", made: "Rust" },
  { part: "Its code generation", made: "LLVM 22" },
  { part: "Its linking", made: "lld 22, through lld’s library and a small C++ shim" },
  { part: "What LLVM’s C API lacks, like custom passes", made: "C++, eventually: a small wrapper, as Rust’s own compiler has" },
  { part: "Reading C headers (`incl.C`)", made: "Clang 22" },
  { part: "The runtime in every program", made: "Rust, `no_std`" },
  { part: "The reference interpreter", made: "Rust" },
  { part: "The checker that tests Zomp against it", made: "Rust" },
  { part: "This website", made: "React Router, Shiki, Cloudflare Workers" },
];

// The comparison table (owner, 2026-10-15: "Latest stable versions of Go, Zig, Rust, C, Zag, Java,
// Python, and C++. Start with 3 first"). Checked on 2026-10-15: versions from the official release
// indexes (static.rust-lang.org's stable channel, ziglang.org/download/index.json); Rust's facts from
// the Rust Book and Reference, Zig's from its 0.17.0 language reference (whose build modes are now
// debug, safe, fast and small, and which translates C with `zig translate-c`, no @cImport); Zag's
// from its design notes. Stated behaviour, not benchmarks or a score.
export const COMPARED = [
  { id: "zag", name: "Zag", version: "In development", url: null },
  { id: "rust", name: "Rust", version: "1.99.0", url: "https://blog.rust-lang.org/2026/10/01/Rust-1.99.0/" },
  { id: "zig", name: "Zig", version: "0.17.0", url: "https://ziglang.org/download/0.17.0/release-notes.html" },
];
export const COMPARISON = [
  { topic: "Maturity", zag: "A hobby language, being experimented with: no release, and no stability to expect.",
    rust: "Stable since 1.0. New editions are opt-in, and older ones keep compiling.",
    zig: "Not yet 1.0: a release can change the language and its standard library." },
  { topic: "Runs as", zag: "Native code: Zomp compiles through LLVM.",
    rust: "Native code, compiled ahead of time through LLVM.",
    zig: "Native code, compiled ahead of time." },
  { topic: "Memory", zag: "Ownership: every value has one owner, and each call says `lend`, `give` or `copy`. No garbage collector.",
    rust: "Ownership and borrowing, checked at compile time; values are dropped with their owner. No garbage collector.",
    zig: "Manual: the language does no memory management for you. Allocators are passed explicitly; there is no default one." },
  { topic: "Overflow and bounds", zag: "Proven at compile time; the final build carries no checks. What can’t be proven is refused until you write a guardrail or `no_check`.",
    rust: "Out-of-bounds indexing panics at run time. Integer overflow panics in debug builds and wraps in release builds by default.",
    zig: "Checked at run time, with a panic, in the debug and safe modes; unchecked in the fast and small modes." },
  { topic: "Errors", zag: "Failures are part of a function’s signature, and the caller must handle each with `otherwise`; forgetting is a compile error.",
    rust: "`Result` values and the `?` operator; ignoring a `Result` is a warning. Panics for the unrecoverable.",
    zig: "Error unions (`!T`) with `try` and `catch`; an error can’t be silently ignored." },
  { topic: "C", zag: "`incl.C` reads C headers while building, with Clang; `extern.C` declares by hand. Every call into C is a `no_check` site.",
    rust: "`extern \"C\"` declarations, called in `unsafe` code; headers are turned into Rust with tools such as bindgen.",
    zig: "Calls C directly; headers are translated with `zig translate-c` or the build system, and Zig can export C libraries." },
];
