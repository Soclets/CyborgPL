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
  signature otherwise. Which library provides it is Synthia's business
  (`Synthia.cplm`), never the source's. One calling convention: C's.
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
  nocheck 'c' := ptr.read['p'];         # the T at 'p'
  nocheck ptr.write['p', 65];               # needs ptr.mut
  nocheck 'q' := ptr.at['p', 3];        # 3 elements further — in T units, never bytes
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
  return (a callback receives pointers); a declaration still cannot hold one.

## A ranged return — C's contract, checked at the door (decided 2026-09-22)

```
extern fn strlen ['s': ptr[uint8]] -> uint64[0, 1_000_000];
nocheck 'n' := strlen[lend 's'] otherwise fail too_long;    # can fail out_of_range
```

An extern's return may be a range type. It is a promise about C that
Cyborg does not trust: the value is compared against the range at the
door, and outside it the call **fails `out_of_range`**, which the caller
handles like any failure. Inside it, Precog knows the value's domain
and everything downstream is provable. It is a failure, not a check
site — so `nocheck` (which the call needs) does not remove it, and the
final build keeps it: one compare per call, the program's own logic,
the same rule as `to.int32[0, 100][…]` at the input boundary. An extern
with a plain return still cannot fail. Parameters cannot be ranged (C
receives what it is given). Proposed: (a) nothing on the extern, a
guard `if` after each call; (b) a trusted range, unverified — a
`nocheck` in disguise, rejected; (c) this. Owner asked the safety and
performance difference against (a): none — same compare, same proofs;
written once in the signature instead of after every call. Owner: c.
`examples/extern_range.cpls`.

## Where C's numbers are checked (owner, 2026-09-27)

A C enum or a range type is checked **wherever it comes in from C**, and
fails `out_of_range` for a number it doesn't allow: a call's return, what
C wrote through a `ptr.mut` you lent it (checked when the call ends), a
`ptr.read`, and a C variable. Each is handled with `otherwise`. Writing the
type is the request to check. To skip the check, write the plain type
(`int32`), and convert when you want the enum:

```
'k' := to.Key['code'] otherwise { out_of_range { return; } };
```

`nocheck` does not remove these. It turns off *check sites*, the bugs
("calling C here is fine"), not *failures*, which are answers you handle.
Returns were ruled so on 2026-09-22. A **callback's** parameter must be the
plain number: C calls it, so a failure would have nowhere to go.

Why it matters: a newer C library adds a case (`KEY_F13 = 302`) that
your binding doesn't list. Checked, that's an `out_of_range` your program
handles. Unchecked, it's undefined behaviour in a `match`. The check is a
comparison or two, under a nanosecond, beside a call into C.

Proposed and not chosen: `nocheck` vouching for the number where it is
already written (b); stopping the program on a bad number (c); a fourth
tier trusting C's answers too. The owner asked whether `nocheck` was then
"lying" (it isn't, for the reason above), then chose this rule.
`examples/c_enum_convert.cpls`.

## A range type as a C struct's field (owner, 2026-09-27: a)

```
layout.c struct Rect { w: int32[0, 4096]  h: int32[0, 4096] }
```

C sees the plain number, so the layout is unchanged (`Rect` is 8 bytes).
The range is Cyborg's promise: proven where Cyborg fills one in, and
checked at the door when a struct comes from C, failing `out_of_range`,
the ranged-return rule applied to fields. Proposed alternative: keep
refusing it, with plain fields checked by hand. A range on an `extern fn`
*parameter* is still refused; that was not asked.
`examples/c_struct_ranges.cpls`.

## `layout.c enum` — C's enum (owner, 2026-09-27: a)

```
layout.c enum Key { a = 65  b = 66  escape = 256 }
extern fn IsKeyDown ['key': Key] -> bool;
```

A C enum is a number with names, and here it is a Cyborg enum laid out
as C's: `match` covers every case, and it is owned like every enum
(2026-09-20). Its cases carry no data, and each says its number, as the C
header does. Guessing the order is how bindings go wrong, so a missing
number is an error, and so are two cases with one number. It is 32 bits,
C's `int`. A number C hands back that matches no case fails `out_of_range`
at the door, like a ranged return: C can send any number. Only a
`layout.c enum` crosses to C. Proposed alternative: no enums at the
boundary, plain numbers with named constants, which loses the names in
types and the exhaustive `match`. Written numbers required, and 32 bits:
design claude's recommendation with a, not asked separately.
`examples/c_enum.cpls`.

## `extern 'x': T` — a C global (decided 2026-09-21; `let` dropped 2026-09-25)

```
extern 'lib_version': int32;        # C: extern int lib_version;   read-only
extern mut 'lib_verbose': int32;    # C: extern int lib_verbose;   may be written

nocheck 'v' := 'lib_version';       # reading C's variable is a nocheck site
nocheck change 'lib_verbose' = 1;              # so is writing one
```

A C library's variables, named the way `extern fn` names its functions.
The type is required and C-shaped (numbers, `bool`, `ptr`, `layout.c`
structs); no value — C owns it; `mut` says it may be written, `immut` is
meaningless. Every read and write is a nocheck site, like an extern call:
nothing about C's memory can be proven. It is a plain value that copies
by itself — no verbs. Proposed alternatives: no extern data at all (a C
helper function per variable) and `extern 'x': ptr[T]` read through
`ptr.read`. Owner chose `extern let`, which lost its `let` with every
other declaration on 2026-09-25.

**`errno` is not a plain global** on macOS and Linux — it is a macro
that calls a function — so `extern 'errno': int32;` would be wrong
there; the std Limb offers `std::c.errno[]` backed by the real thing.
`extern 'x': T` is for genuinely plain C globals. `examples/extern_data.cpls`.

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
  register into `'v'` after (declaring it, like `'v' := …`), `'v' <-> x0`
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

- Reading a C string into a `str.utf8` — std's, behind one intrinsic.
- A text's length (`list.length` refuses a `str`) — string operations are
  open; `strlen` through the door is the stopgap.
