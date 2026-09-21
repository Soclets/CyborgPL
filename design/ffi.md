# Low level — `extern`, pointers, C layout, `size.of`

Decided 2026-09-19, one question at a time, after the owner asked how
std's print gets made and then: "Syscall and lower-level stuff should be
able to do, using the base Cyborg language." So the door to the OS is the
language's, open to every Limb (as in C and Rust); std is the library
everyone uses, not a privileged one. Inline assembly is also in — spelling
not yet designed. In the oracle since 2026-09-20: every rule below is
enforced; the oracle cannot link C, so it emulates `write`, `exit`,
`getenv`, `strlen`, `malloc`, `free` (`examples/ffi.cpls`).

## `extern` — a function whose body is somewhere else

```
extern fn write ['fd': int32, 'buf': ptr[uint8], 'len': uint64] -> int64;
extern fn exit ['code': int32];
extern fn printf ['fmt': ptr[uint8], ...'args'] -> int32;      # C's own varargs
```

- `extern` before `fn`, like `pub`; no body, a `;` instead. A normal
  signature otherwise. Which library provides it is JetPack's business
  (`JetPack.toml`), never the source's. One calling convention: C's.
- **An `extern` call is a `nocheck` site** — Precog cannot see inside it,
  so nothing about it can be proven; `nocheck` already means "on my
  word", and the build output already lists every such site, so it now
  lists every place a program leaves the language. No second `unsafe`
  word. The mark lives in wrappers: std's `print` has one `nocheck`
  line; user programs calling `std::print` have none.
- A returned plain value is a boundary value (its type known, its range
  not) — the wrapper's `if 'n' < 0 { fail io; }` makes it ordinary.
- Variadic externs use `...'args'`; each extra argument must be C-shaped;
  C's promotions apply (`bin32` → `bin64`, small ints → `int32`); the
  platform's varargs convention (Apple arm64 differs) is the compiler's.

## `ptr[T]` — exists only at the boundary

- `ptr[T]` (read-only) and `ptr.mut[T]` may appear in an `extern`
  signature and in a `layout.c` struct, nowhere else as a declared type.
- **A lend satisfies a pointer**: `write[1, lend 'bytes', 3]` hands C the
  address of the lent data; `lend.mut` for `ptr.mut`. The borrow rules
  are the pointer's validity: the value is alive and unaliased for the
  duration of the call.
- A pointer C hands back may be **held and passed back** (a `FILE*`
  handle), and — owner's choice (b) over opaque handles — **read and
  written through**:

  ```
  nocheck let 'c' := ptr.read['p'];         # the T at 'p'
  nocheck ptr.write['p', 65];               # needs ptr.mut
  nocheck let 'q' := ptr.at['p', 3];        # 3 elements further — in T units, never bytes
  if ptr.is_null['f'] { fail not_found; }   # C's null; reading it would crash
  ```
  All `nocheck` sites. No pointer↔integer conversion; a pointer comes
  only from a lend or from C.
- No raw-allocation primitive: bytes you can name are a `list[uint8]` /
  `array[uint8]` (owned, checked, lent to C); bytes nobody owns are
  `malloc`/`free`, two externs.

## Callbacks — a Cyborg function C calls (decided 2026-09-21)

```
extern fn qsort ['base': ptr.mut[int32], 'n': uint64, 'size': uint64,
                 'cmp': fn[ptr[int32], ptr[int32]] -> int32];

fn ascending ['a': ptr[int32], 'b': ptr[int32]] -> int32 {
    nocheck return ptr.read['a'] - ptr.read['b'];
}
nocheck qsort[lend.mut 'nums', 5, size.of[int32], ascending];
```

- **Only capture-free functions cross** (owner: a, over smuggling captures
  through a `void *userdata`). A C callback slot holds an address and
  nothing else; a function with `with [lend 'limit']` handed to C is a
  compile error — "C has nowhere to keep `'limit'`" — and the userdata
  pattern is done by hand with an ordinary `ptr[…]` argument.
- In an `extern` signature a callback is a `fn[…]` type held to the
  C-shaped rule through and through: parameters by value (no `lent`; C
  passes pointers), C-shaped types, no `fail`. The compiler emits the
  function with C's convention when it is handed over.
- A function pointer C hands *to* Cyborg (`signal` returns the previous
  handler) is a `fn[…]` value to hold and pass back; calling it is an
  `extern` call, a `nocheck` site.
- Consequence: `ptr[T]` may now appear in any function's parameters and
  return (a callback receives pointers); a `let` still cannot declare one.

## `layout.c` — a struct in C's shape

```
layout.c struct Timeval {
    sec: int64
    usec: int64
}
```

Only a `layout.c` struct may cross to C (in a signature or a `ptr[…]`); a
normal struct's layout is Precog's. Its fields must be C-shaped: numbers,
`bool`, `ptr[…]`, other `layout.c` structs — no `str`, no `list`.

## `size.of[T]`, `align.of[T]`

Compile-time constants, spelled like `to.`: `malloc[10 x size.of[Timeval]]`.
A `layout.c` struct has C's size; a normal struct's is whatever Precog
chose.

## Inline assembly — `inline.ASM` (decided 2026-09-20)

```
nocheck inline.ASM {
    mov x16, #4
    svc 0
} with ['fd' -> x0, 'buf' -> x1, 'len' -> x2, 'n' <- x0, clobber x16];
```

- **`inline.<LANGUAGE>`** — owner's shape, over bare `asm`: a family, with
  assembly its first member; `inline.C` is feasible later without new
  machinery (LLVM ships clang). **The language's name is in capitals, like
  `MAIN`**, so an inline block is easy to spot (owner, over Claude's
  lowercase `inline.asm`). `inline.asm` is refused with that reason.

- **A statement, not a value** (owner: option 1) — a syscall can set
  several registers, so results come out through named bindings.
- **The braces hold raw assembly, verbatim**, for the target CPU (arm64
  on the owner's Mac: `mov x16, #4` / `svc 0` is macOS's `write`); the
  compiler does not read it, LLVM's assembler does. Not a list of strings
  (owner: "I don't think Assembly looks like that").
- **`with [ … ]` binds variables to registers; the arrow is the
  direction** (owner: "drop the words, keep the arrows"): `'v' -> x0`
  copies the variable in before the block, `'v' <- x0` stores the
  register into `'v'` after (declaring it, like a `let`), `'v' <-> x0`
  both; `clobber x16` says a register was overwritten.
- **Compiler rule (2026-09-21):** on AArch64 the lowering spells `x29` as
  `fp` and `x30` as `lr` in what it hands LLVM — the ABI names. LLVM 22
  ignores a clobber of those two when written as `xN` (fixed in 23; Clang
  never hit it because it always emits the ABI names). The programmer
  keeps writing `x30`; the rename is invisible and harmless on 23.
- Always a `nocheck` site. Per-architecture by nature.
- The oracle parses and checks all of this and then refuses to run it
  ("the oracle cannot run assembly"), with nine reject tests.

## Open

- Extern data (`extern const`/`let` for a C global).
- Reading a C string into a `str.utf8` — std's, behind one intrinsic.
- A text's length (`list.length` refuses a `str`) — string operations are
  open; `strlen` through the door is the stopgap.
