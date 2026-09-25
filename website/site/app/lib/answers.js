// Instant answers: type one of `keys` exactly and this card comes first.
// Text is from design/ (ownership-syntax.md, failures.md, concurrency.md) and design
// claude. Every `code` line is copied verbatim from examples/<from>.cpls — the build
// (scripts/search-index.mjs) fails if it isn't.
export const ANSWERS = [
  { keys: [":="], title: ":= works out the type", text: "Declares a value and picks its best type from the whole program. A value from outside needs its type written instead.", code: "let 'i' := 1;", from: "fizzbuzz", to: "/learn#syntax-2" },
  { keys: ["let", "="], title: "let declares", text: "let 'name' := value; works the type out. let 'name': type = value; writes it.", code: "let immut 'limit': int32 = 15;", from: "fizzbuzz", to: "/learn#syntax-2" },
  { keys: ["x", "multiply", "times"], foreign: ["*"], title: "x multiplies", text: "x multiplies, xx is power, mod is the remainder.", code: "const 'twice_max' := 'max' x 2;", from: "consts", to: "/learn#syntax-4" },
  { keys: ["xx", "power", "pow", "exponent"], foreign: ["**"], title: "xx is power", text: "x multiplies, xx is power, mod is the remainder.", code: "std::printu.stdout[(2 xx 3 xx 2), \\n];", from: "power", to: "/learn#syntax-4" },
  { keys: ["mod", "modulo", "remainder"], foreign: ["%"], title: "mod is the remainder", text: "x multiplies, xx is power, mod is the remainder.", code: "let 'by3' := ('n' mod 3) == 0;", from: "fizzbuzz", to: "/learn#syntax-4" },
  { keys: ["++", "concat", "concatenate", "join"], title: "++ joins text", text: "++ joins text: at least one side is text, the other is written out.", code: "claimed 'who' { return \"taken by \" ++ 'who'; }", from: "enums", to: "/examples/enums" },
  { keys: ["xor", "and.bit", "or.bit", "not.bit", "shl", "shr", "bitwise"], foreign: ["&", "|", "~", "<<", ">>"], title: "Bitwise is spelled out", text: "and.bit, or.bit, not.bit, xor, shl, shr — the boolean words, per bit.", code: "std::printu.stdout['a' xor 'b', \\n];", from: "bitwise", to: "/examples/bitwise" },
  { keys: ["pos", "index", "indexing"], title: "pos is a position", text: "Positions count from 1: [pos 1] is the first element.", code: "return 'xs'[pos 1];", from: "generics", to: "/learn#syntax-5" },
  { keys: ["\\n", "newline"], title: "\\n is the newline", text: "A string is exactly its characters. The newline is its own piece, outside the quotes.", code: "std::print.stdout[\"Hello, World!\", \\n];", from: "hello", to: "/learn#syntax-6" },
  { keys: ["lend", "lend.mut", "borrow"], title: "lend borrows", text: "The receiver borrows the value and the owner keeps it. lend.mut borrows it mutably.", code: "std::printu.stdout[\"#\", 'l'.number, \": \", describe[lend 'l'], \\n];", from: "enums", to: "/learn#ownership" },
  { keys: ["give", "move"], title: "give moves", text: "Ownership moves to the receiver.", code: "let 't' := give 's';", from: "ownership", to: "/learn#ownership" },
  { keys: ["copy"], title: "copy duplicates", text: "The receiver gets its own copy; the original stays.", code: "let 'u' := copy 't';", from: "ownership", to: "/learn#ownership" },
  { keys: ["fail", "otherwise"], title: "fail and otherwise", text: "A function lists how it can fail in its signature (-> T or fail[…]); the caller must say what happens, with otherwise.", code: "let 'f' := first[lend 'nums'] otherwise 0;", from: "generics", to: "/learn#failures" },
  { keys: ["exception", "exceptions", "try", "catch", "throw"], title: "No exceptions", text: "CyborgPL has no exceptions. A function that can fail says so in its signature, and the caller must handle it — forgetting is a compile error.", code: "let 'f' := first[lend 'nums'] otherwise 0;", from: "generics", to: "/learn#failures" },
  { keys: ["gc", "garbage collector", "garbage"], title: "No garbage collector", text: "Every value has one owner, so the compiler knows when it can be freed.", code: "let 't' := give 's';", from: "ownership", to: "/learn#ownership" },
  { keys: ["concur", "async", "thread", "threads"], title: "concur runs alongside", text: "concur { … } runs a block alongside the rest. Channels pass values between tasks.", code: "concur with [lend 'results'] { channel.send[lend 'results', fetch[\"b.com\"]]; }", from: "concur", to: "/examples/concur" },
  { keys: ["channel", "channels"], title: "channel", text: "Passes values between concurrent tasks — a value is given, never lent.", code: "let 'results' := channel[str.utf8];", from: "concur", to: "/examples/concur" },
  { keys: ["fn", "function", "functions"], title: "fn declares a function", text: "Parameters go in [ ], the return type after ->. Calls use brackets too: add[1, 2].", code: "fn add ['a': int32, 'b': int32] -> int32 {", from: "add", to: "/examples/add" },
  { keys: ["match", "switch"], title: "match picks by case", text: "Chooses by enum case; a case can bind what it carries.", code: "match 'l'.status {", from: "enums", to: "/examples/enums" },
  { keys: ["main", "entry", "entry point"], title: "MAIN is where it starts", text: "A program starts in its MAIN block.", code: "MAIN {\n    std::print.stdout[\"Hello, World!\", \\n];\n}", from: "hello", to: "/examples/hello" },
  { keys: ["range"], title: "range counts", text: "range[a, b] counts from a to b, both included.", code: "for 'i' in range[1, 'count'] { list.append[lend.mut 'out', 'i' x 'i']; }", from: "bounded", to: "/examples/bounded" },
];

// `foreign` are other languages' spellings: built-in, they mean nothing here
// (design/operators.md — `*`, `%`, `&`… are free for a program to declare).
export const ANSWER_BY_KEY = new Map(ANSWERS.flatMap((a, i) => [...a.keys, ...(a.foreign ?? [])].map((k) => [k, i])));
