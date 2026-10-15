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
// from its design notes. Then C, C++ and Go ("Do 3 more now"): C23 (ISO/IEC 9899:2024) and C++23
// (ISO/IEC 14882:2024) are the latest published standards (C2y is in progress; C++26 is finished but
// its ISO ballot runs to about late October 2026), facts from cppreference; Go 1.27.2 from go.dev's
// release index, facts from the Go spec, the Go 1 compatibility promise and the cgo docs. Then Java
// and Python ("yea"): JDK 27, generally available on 2026-09-15 (openjdk.org), facts from the Java
// Language Specification SE 27 and JEP 454; Python 3.15.0, released 2026-10-09 (python.org), facts
// from its 3.15 documentation, meaning CPython.
// Stated behaviour, not benchmarks or a score.
export const COMPARED = [
  { id: "zag", name: "Zag", version: "In development", url: null },
  { id: "rust", name: "Rust", version: "1.99.0", url: "https://blog.rust-lang.org/2026/10/01/Rust-1.99.0/" },
  { id: "zig", name: "Zig", version: "0.17.0", url: "https://ziglang.org/download/0.17.0/release-notes.html" },
  { id: "c", name: "C", version: "C23", url: "https://en.cppreference.com/w/c/23" },
  { id: "cpp", name: "C++", version: "C++23", url: "https://en.cppreference.com/w/cpp/23" },
  { id: "go", name: "Go", version: "1.27.2", url: "https://go.dev/doc/go1.27" },
  { id: "java", name: "Java", version: "JDK 27", url: "https://openjdk.org/projects/jdk/27/" },
  { id: "python", name: "Python", version: "3.15.0", url: "https://www.python.org/downloads/release/python-3150/" },
];
export const COMPARISON = [
  { topic: "Maturity", zag: "A hobby language, being experimented with: no release, and no stability to expect.",
    rust: "Stable since 1.0. New editions are opt-in, and older ones keep compiling.",
    zig: "Not yet 1.0: a release can change the language and its standard library.",
    c: "An ISO standard since 1989; C23 is the latest edition, and compilers keep older ones.",
    cpp: "An ISO standard since 1998; C++23 is the latest published edition, and old code keeps compiling.",
    go: "Stable: Go 1 promises that code keeps compiling with later 1.x releases, with documented exceptions.",
    java: "Stable, with strong compatibility; a release every six months, and a long-term-support one every two years.",
    python: "Stable: a release every year, and old behaviour is deprecated before it is removed.", },
  { topic: "Runs as", zag: "Native code: Zomp compiles through LLVM.",
    rust: "Native code, compiled ahead of time through LLVM.",
    zig: "Native code, compiled ahead of time.",
    c: "Native code, compiled ahead of time by GCC, Clang, MSVC and others.",
    cpp: "Native code, compiled ahead of time by GCC, Clang, MSVC and others.",
    go: "Native code, with Go’s runtime built in for garbage collection and goroutines.",
    java: "Bytecode on the JVM, which compiles busy code to native code as it runs.",
    python: "CPython runs bytecode in an interpreter, with an experimental JIT.", },
  { topic: "Memory", zag: "Ownership: every value has one owner, and each call says `lend`, `give` or `copy`. No garbage collector.",
    rust: "Ownership and borrowing, checked at compile time; values are dropped with their owner. No garbage collector.",
    zig: "Manual: the language does no memory management for you. Allocators are passed explicitly; there is no default one.",
    c: "Manual: `malloc` and `free`, with no garbage collector.",
    cpp: "Manual, made safer by destructors (RAII) and smart pointers like `unique_ptr`. No garbage collector.",
    go: "A garbage collector frees what’s no longer reachable.",
    java: "A garbage collector frees what’s no longer reachable.",
    python: "Reference counting, with a garbage collector for cycles.", },
  { topic: "Overflow and bounds", zag: "Proven at compile time; the final build carries no checks. What can’t be proven is refused until you write a guardrail or `no_check`.",
    rust: "Out-of-bounds indexing panics at run time. Integer overflow panics in debug builds and wraps in release builds by default.",
    zig: "Checked at run time, with a panic, in the debug and safe modes; unchecked in the fast and small modes.",
    c: "Unchecked: signed overflow and out-of-bounds access are undefined behaviour; unsigned numbers wrap.",
    cpp: "Unchecked by default: signed overflow is undefined behaviour and `[]` doesn’t check bounds; `.at()` does, throwing an exception.",
    go: "Out-of-range indexing panics at run time. Integer overflow wraps without a panic, as the spec defines.",
    java: "Out-of-bounds indexing throws an exception. Integer overflow wraps silently; `Math.addExact` and kin throw instead.",
    python: "Integers grow as needed, so they don’t overflow. Out-of-range indexing raises `IndexError`.", },
  { topic: "Errors", zag: "Failures are part of a function’s signature, and the caller must handle each with `otherwise`; forgetting is a compile error.",
    rust: "`Result` values and the `?` operator; ignoring a `Result` is a warning. Panics for the unrecoverable.",
    zig: "Error unions (`!T`) with `try` and `catch`; an error can’t be silently ignored.",
    c: "Return values and `errno`; nothing makes the caller check them.",
    cpp: "Exceptions, and since C++23 `std::expected` for errors returned as values.",
    go: "Errors are returned as values of the `error` type and checked with `if`; `panic` for the unrecoverable.",
    java: "Exceptions; a checked exception must be caught or declared, or the code doesn’t compile.",
    python: "Exceptions; nothing makes the caller handle them.", },
  { topic: "C", zag: "`incl.C` reads C headers while building, with Clang; `extern.C` declares by hand. Every call into C is a `no_check` site.",
    rust: "`extern \"C\"` declarations, called in `unsafe` code; headers are turned into Rust with tools such as bindgen.",
    zig: "Calls C directly; headers are translated with `zig translate-c` or the build system, and Zig can export C libraries.",
    c: "It is C.",
    cpp: "Includes C headers and calls C directly, with `extern \"C\"` for linking.",
    go: "Through cgo: C declared in a comment above `import \"C\"`; each call has some overhead.",
    java: "The Foreign Function & Memory API calls C without JNI; `jextract` turns headers into Java.",
    python: "`ctypes` in the standard library calls C libraries; extension modules are written against the C API.", },
];

// Speed (owner, 2026-10-15: "yea", to benchmarks on Home, then "rerun"). From bench/RESULTS.md, the
// run of 2026-10-15, the first under the name Zag (Zomp's release build of ab80b214, 10 threads
// and a 1-thread row, everything else paused). Seconds, best of 3, wall-clock from start to exit;
// every answer checked against C's. Python is left off the charts and given in words. The all-cores
// bars are not like for like (owner, 2026-10-06: "note the nuance").
export const BENCH = {
  date: "15 Oct 2026",
  bars: [
    { id: "zag1", name: "Zag, one core", kind: "ours" },
    { id: "c", name: "C", kind: "other" },
    { id: "rust", name: "Rust", kind: "other" },
    { id: "go", name: "Go", kind: "other" },
    { id: "zagN", name: "Zag, all cores", kind: "ours-all" },
  ],
  programs: [
    { id: "sum", what: "Adds up i × i mod 7 for 300 million numbers", times: { zag1: 0.092, c: 0.096, rust: 0.173, go: 0.143, zagN: 0.021 }, python: 16.433 },
    { id: "fib", what: "Recursive Fibonacci of 35: all function calls", times: { zag1: 0.016, c: 0.017, rust: 0.016, go: 0.023, zagN: 0.017 }, python: 0.516 },
    { id: "lists", what: "Appends 20 million numbers to a list, then sums it 10 times", times: { zag1: 0.040, c: 0.042, rust: 0.052, go: 0.105, zagN: 0.036 }, python: 8.788 },
  ],
  // sieve is held back (design claude, 2026-10-15): in this run its final build failed on a proof
  // regression in Zomp, and bench/run.py fell back to a working build with a bounds check in the hot
  // loop. It comes back when RESULTS.md has the re-run after compiler claude's fix.
  heldBack: "sieve",
};
