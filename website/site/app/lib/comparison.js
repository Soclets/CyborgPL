// Checked against the project and official language documentation on 2026-10-03.
// These are stated behaviors and costs, not benchmark results or a winner score.
import { REPO_URL } from "./content.js";

const repo = (file) => `${REPO_URL}/blob/main/${file}`;
const fact = (text, url, label = "Docs", extra = []) => ({ text, sources: [{ url, label }, ...extra] });

export const COMPARISON_LANGUAGES = [
  { id: "nerve", name: "Nerve", version: null, versionUrl: null, status: "In development" },
  { id: "rust", name: "Rust", version: "1.99.0", versionUrl: "https://blog.rust-lang.org/2026/10/01/Rust-1.99.0/" },
  { id: "go", name: "Go", version: "1.27.1", versionUrl: "https://go.dev/doc/devel/release#go1.27.0" },
  { id: "python", name: "Python", version: "CPython 3.14.8", versionUrl: "https://www.python.org/downloads/release/python-3148/" },
];

export const COMPARISON_ROWS = [
  { id: "status", title: "Maturity & stability", cells: {
    nerve: fact("A hobby language in active development. No packaged release or stability promise; the compiler and libraries still have gaps.", repo("README.md"), "Project", [{ label: "Stability", url: repo("DECISIONS.md") }]),
    rust: fact("A released, stable toolchain. Edition changes are opt-in; the compiler continues supporting older editions.", "https://doc.rust-lang.org/book/appendix-05-editions.html"),
    go: fact("A released, stable toolchain with the Go 1 source-compatibility promise and documented exceptions.", "https://go.dev/doc/go1compat"),
    python: fact("A released CPython implementation. Public APIs follow a compatibility and deprecation policy, with exceptions for issues such as security fixes.", "https://peps.python.org/pep-0387/"),
  } },
  { id: "execution", title: "Execution", cells: {
    nerve: fact("CCA compiles supported programs to native code through LLVM. The oracle is a separate reference interpreter.", repo("cca/README.md"), "Compiler"),
    rust: fact("Ahead-of-time compilation: the resulting executable can run without the Rust toolchain installed.", "https://doc.rust-lang.org/book/ch01-02-hello-world.html"),
    go: fact("Compilation to machine code. Go programs include runtime support for facilities such as garbage collection and goroutines.", "https://go.dev/doc/faq#runtime"),
    python: fact("CPython normally interprets bytecode. Native extensions and an optional experimental JIT can carry out work in machine code.", "https://docs.python.org/3.14/glossary.html#term-bytecode", "Docs", [{ label: "Experimental JIT", url: "https://docs.python.org/3.14/library/sys.html#sys._jit" }]),
  } },
  { id: "types", title: "Types", cells: {
    nerve: fact("Static types. The design asks Precog to choose a safe, efficient type across the program with `:=`; outside boundaries need explicit types. Inference is still evolving.", repo("design/core-features.md"), "Design"),
    rust: fact("Static types with inference, generics and traits. Ownership and borrowing constraints are checked at compile time.", "https://doc.rust-lang.org/book/ch04-01-what-is-ownership.html", "Ownership", [{ label: "Types", url: "https://doc.rust-lang.org/book/ch10-00-generics.html" }]),
    go: fact("Static types with inference, interfaces and generics. Memory management does not use an ownership or borrow checker.", "https://go.dev/ref/spec#Types"),
    python: fact("Dynamic types. Annotations support static checking tools, but Python itself does not enforce function and variable type hints.", "https://docs.python.org/3.14/library/typing.html"),
  } },
  { id: "memory", title: "Memory", cells: {
    nerve: fact("Ownership with explicit `lend`, `give` and `copy`. The compiler arranges reclamation; the model does not require tracing garbage collection.", repo("design/ownership-syntax.md"), "Design"),
    rust: fact("Ownership and borrowing, with values dropped when their owner is dropped. No mandatory garbage collector; reference counting is available.", "https://doc.rust-lang.org/book/ch04-01-what-is-ownership.html", "Ownership", [{ label: "Reference counting", url: "https://doc.rust-lang.org/book/ch15-04-rc.html" }]),
    go: fact("Automatic garbage collection. Collection uses CPU and additional heap memory; its behavior can be tuned.", "https://go.dev/doc/gc-guide"),
    python: fact("Automatic memory management: CPython combines reference counting with cyclic garbage collection. Other Python implementations can use different collectors.", "https://docs.python.org/3.14/library/gc.html", "GC", [{ label: "Implementation", url: "https://docs.python.org/3.14/reference/datamodel.html" }]),
  } },
  { id: "checks", title: "Safety checks", cells: {
    nerve: fact("Precog proves supported checks. Working builds keep unproven runtime checks; final builds refuse them unless explicitly trusted. Compiler and proof bugs are still being found.", repo("design/core-features.md"), "Design", [{ label: "Known proof bug", url: repo("check/corpus/2026-10-03-precog-value-set-misses-a-call.cpls") }]),
    rust: fact("Compile-time ownership checks in safe code; bounds checks can remain at runtime. Ordinary integer overflow is checked in debug builds and normally wraps in release builds.", "https://doc.rust-lang.org/book/ch03-02-data-types.html"),
    go: fact("Out-of-range indexing panics. Fixed-width integer overflow is allowed and follows the language's integer rules; overflow itself does not panic.", "https://go.dev/ref/spec#Integer_overflow"),
    python: fact("Integers grow to the available memory rather than overflowing a fixed width. Out-of-range list indexing raises `IndexError`; type errors can occur at runtime.", "https://docs.python.org/3.14/library/stdtypes.html"),
  } },
  { id: "failures", title: "Failure handling", cells: {
    nerve: fact("Function signatures list `fail` outcomes. Callers must handle them with `otherwise`, including forwarding a failure to their own caller.", repo("design/failures.md"), "Design"),
    rust: fact("`Result` and `?` handle recoverable errors; panics also exist. Ignoring a `Result` produces a warning by default, which projects can make an error.", "https://doc.rust-lang.org/std/result/"),
    go: fact("`error` is an ordinary return value. Handling is usually explicit, but the compiler does not force every error to be handled. `panic` and `recover` also exist.", "https://go.dev/doc/faq#exceptions"),
    python: fact("Exceptions with `try` / `except`. The language does not require callers to enumerate and handle every possible exception.", "https://docs.python.org/3.14/tutorial/errors.html"),
  } },
  { id: "concurrency", title: "Concurrency", cells: {
    nerve: fact("`concur` and channels, with a prototype runtime multiplexing tasks over worker threads. Its guarantees still depend on the developing compiler and runtime.", repo("cca/runtime/src/tasks.rs"), "Runtime"),
    rust: fact("Threads, channels and async/await. Ownership with `Send` and `Sync` prevents data races in sound safe code; deadlocks and logical races remain possible.", "https://doc.rust-lang.org/book/ch16-00-concurrency.html"),
    go: fact("Goroutines, channels and synchronization tools. Data races are possible; the race detector checks executed paths, not every possible execution.", "https://go.dev/doc/articles/race_detector"),
    python: fact("Threads, `asyncio` and processes. Default CPython has a GIL; optional free-threaded builds allow parallel Python threads, with extension compatibility caveats.", "https://docs.python.org/3.14/howto/free-threading-python.html"),
  } },
  { id: "tooling", title: "Everyday tooling", cells: {
    nerve: fact("CCA and the oracle are in the repo. Library coverage and supporting tools are developing; there is no packaged download yet.", repo("README.md"), "Project"),
    rust: fact("Cargo handles builds, dependencies and tests, with documentation and formatting tools. Packages are distributed through crates.io.", "https://doc.rust-lang.org/cargo/"),
    go: fact("The `go` tools cover building, testing, modules, formatting, vetting and race detection.", "https://pkg.go.dev/cmd/go"),
    python: fact("The interpreter, `venv`, `pip` and PyPI support development and packages. Projects choose their testing and static checking tools.", "https://docs.python.org/3.14/tutorial/venv.html"),
  } },
  { id: "costs", title: "Practical costs", cells: {
    nerve: fact("Every build includes Precog. The design spends compile time to remove runtime checks; changing syntax and missing implementation pieces also cost developer time.", repo("design/core-features.md"), "Design"),
    rust: fact("Borrowing and lifetime constraints shape API design. `unsafe` code and foreign interfaces put additional safety obligations on their authors.", "https://doc.rust-lang.org/book/ch20-01-unsafe-rust.html"),
    go: fact("GC trades CPU and memory for automatic reclamation. Synchronization and explicit error handling still require attention.", "https://go.dev/doc/gc-guide"),
    python: fact("Without extra checking, some errors surface only when their path runs. CPU parallelism depends on the build, extensions, and whether work uses threads or processes.", "https://docs.python.org/3.14/library/threading.html", "Threads", [{ label: "Type checking", url: "https://docs.python.org/3.14/library/typing.html" }]),
  } },
];
