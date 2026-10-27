# Low level — `extern`, pointers, C layout, `size.of`

> Revised 2026-10-07: code here is in the new syntax; an old spelling in the prose is history. The old → new table is `revision-2026-10-07.md`.

> Since 2026-10-10 everything about C here (`extern.C`, `layout.C`, callbacks, and `incl.C`, C headers) is the CNC limb's (`cores.md`): a kit lists CNC to use it. `ptr`, raw memory and inline assembly stay the base's.

Decided 2026-09-19, one question at a time, after the owner asked how
std's print gets made and then: "Syscall and lower-level stuff should be
able to do, using the base Cyborg language." So the door to the OS is the
language's, open to every kit (as in C and Rust); std is the library
everyone uses, not a privileged one. Inline assembly is also in — spelling
not yet designed. In the oracle since 2026-09-20: every rule below is
enforced; the oracle cannot link C, so it emulates `write`, `exit`,
`getenv`, `strlen`, `malloc`, `free` (`examples/ffi.cy`).

## `extern.C` — a function whose body is somewhere else

```
%extern.C proc write [let $fd: int.sign.32, let $buf: ptr[int.unsign.8], let $len: int.unsign.64] -> int.sign.64;
%extern.C proc exit [let $code: int.sign.32];
%extern.C proc printf [let $fmt: ptr[int.unsign.8], let ...$args] -> int.sign.32;      # C's own varargs

%extern.C:                                  # a library's worth at once
    proc malloc [let $bytes: int.unsign.64] -> ptr.mut[int.unsign.8];
    proc free [let $p: ptr.mut[int.unsign.8]];
    $lib_version: int.sign.32;
    var $lib_verbose: int.sign.32;
end of %extern.C
```

**`extern` names its language, `extern.C`, and may be a block** (owner,
2026-10-02: "both", then "`extern.C`, and one line too"). One line, or a
block of `fn` lines and C globals with `extern.C` written once; both stay.
The `.C` is every language's capitalized name, as `layout.C`; a bare
`extern fn` and `extern.c` are refused. Design claude had recommended a
plain `extern { … }`, since `extern` reaches only C. **Each line may say
its own `pub` / `pub.kit`, and a `pub` before the block overrides them
all** (owner, 2026-10-02: "allow mixing, but block overrides"): in a plain
`extern.C { … }` a line is private unless it says `pub fn …`; in
`pub extern.C { … }` every line is public. One-line: `pub extern.C fn …`.
`examples/low_level.cy`, `examples/extern_block_pub.cy`.

- `extern` before `fn`, like `pub`; no body, a `;` instead. A normal
  signature otherwise. Which library provides it is Synthia's business
  (the manifest (`Synthia.ttsr` / `CyJet.cyr`)), never the source's. One calling convention: C's.
- **An `extern` call is a `no_check` site** — the prover cannot see inside it,
  so nothing about it can be proven; `no_check` already means "on my
  word", and the build output already lists every such site, so it now
  lists every place a program leaves the language. No second `unsafe`
  word. The mark lives in wrappers: std's `print` has one `no_check`
  line; user programs calling `std::print` have none.
- A returned plain value is a boundary value (its type known, its range
  not) — the wrapper's `if 'n' < 0 { fail io; }` makes it ordinary.
- Variadic externs use `...'args'`; each extra argument must be C-shaped;
  C's promotions apply (`f32` → `f64`, small ints → `i32`); the
  platform's varargs convention (Apple arm64 differs) is the compiler's.

## `ptr[T]` — exists only at the boundary

- `ptr[T]` (read-only) and `ptr.mut[T]` may appear in an `extern`
  signature and in a `layout.C` struct, nowhere else as a declared type.
- **A lend satisfies a pointer**: `write[1, lend 'bytes', 3]` hands C the
  address of the lent data; `lend.mut` for `ptr.mut`. The borrow rules
  are the pointer's validity: the value is alive and unaliased for the
  duration of the call.
- A pointer C hands back may be **held and passed back** (a `FILE*`
  handle), and — owner's choice (b) over opaque handles — **read and
  written through**:

  ```
  no_check let $c := ptr.read($p);       # the T at 'p'
  no_check ptr.write($p, value: 65);               # needs ptr.mut
  no_check let $q := ptr.at($p, by: 3);      # 3 elements further — in T units, never bytes
  if ptr.is_null($f) { fail not_found; }   # C's null; reading it would crash
  ```
  All `no_check` sites. No pointer↔integer conversion; a pointer comes
  only from a lend, from C, or as `ptr.null`.
- **`ptr.null` is C's `NULL`** (owner, 2026-10-21: "yea, add it", then "widened (a)"): the pointer to
  nothing, a value, not a call. It goes only where its pointer type is known: a C call's argument
  (`strtol[lend 's', ptr.null, 10]`, `free[ptr.null]`), a `layout.C` field, a list beside pointers of a
  known kind, which come from C since a list cannot hold a lend (`['home', ptr.null]`, as `execv` wants). A `:=`
  holding nothing but it, alone or as every item of a list, is refused: nothing would say what it points
  at. It satisfies `ptr.mut` too; reading or writing through it is a `no_check` site like any other, and
  fails. `ptr.is_null` asks; `ptr.null` is the answer's value. `examples/ptr_null.cy`.
- **Symbols, old since 2026-10-16** (they are not operators, `operators.md`), refused naming the word;
  from 2026-10-14 (owner): `*const T` was `ptr[T]`, `*mut T`
  `ptr.mut[T]` (Rust's raw-pointer spelling), `*'p'` reads (`ptr.read['p']`)
  and `change *'p' = v;` writes (`ptr.write`), each still a `no_check` site.
- **`ptr.cast[T]['p']` sees the same address as pointing at a `T`**
  (owner, 2026-09-30: a): `malloc`'s `ptr.mut[u8]` becomes a
  `ptr.mut[Point]`. No byte moves and nothing converts — `ptr.read` then
  reads a `T`, `ptr.at` steps whole `T`s. Always a `no_check` site: only
  the programmer can say the bytes really are a `T`. The target is
  C-shaped, as anything a pointer points at. **A cast keeps whether the
  pointer may write** (owner: a): a `ptr` stays read-only, a `ptr.mut`
  stays writable. It compiles to nothing — LLVM has one pointer type.
  `examples/ptr_cast.cy`.
- No raw-allocation primitive: bytes you can name are a `list[u8]` /
  `array[u8]` (owned, checked, lent to C); bytes nobody owns are
  `malloc`/`free`, two externs.

## Callbacks — a Cyborg function C calls (decided 2026-09-21)

```
%extern.C proc qsort [let $base: ptr.mut[int.sign.32], let $n: int.unsign.64, let $size: int.unsign.64,
                 let $cmp: fn[ptr[int.sign.32], ptr[int.sign.32]] -> int.sign.32];

proc ascending [let $a: ptr[int.sign.32], let $b: ptr[int.sign.32]] -> int.sign.32:
    !no_check ret ptr.read($a) minus ptr.read($b);
end of proc ascending
!no_check qsort(lend.mut $nums, 5, size.of[int.sign.32], ascending);
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
  `extern` call, a `no_check` site.
- Consequence: `ptr[T]` may now appear in any function's parameters and
  return (a callback receives pointers); a declaration still cannot hold one.

## A ranged return — C's contract, checked at the door (decided 2026-09-22)

```
%extern.C proc strlen [let $s: ptr[int.unsign.8]] -> int.unsign.64[min: 0, max: 1_000_000];
!no_check let $n := strlen(lend $s) otherwise fail too_long;    # can fail out_of_range
```

An extern's return may be a range type. It is a promise about C that
Cyborg does not trust: the value is compared against the range at the
door, and outside it the call **fails `out_of_range`**, which the caller
handles like any failure. Inside it, the prover knows the value's domain
and everything downstream is provable. It is a failure, not a check
site — so `no_check` (which the call needs) does not remove it, and the
final build keeps it: one compare per call, the program's own logic,
the same rule as `to.i32[min: 0, max: 100][…]` at the input boundary. An extern
with a plain return still cannot fail. Parameters cannot be ranged (C
receives what it is given). Proposed: (a) nothing on the extern, a
guard `if` after each call; (b) a trusted range, unverified — a
`no_check` in disguise, rejected; (c) this. Owner asked the safety and
performance difference against (a): none — same compare, same proofs;
written once in the signature instead of after every call. Owner: c.
`examples/extern_range.cy`.

## Where C's numbers are checked (owner, 2026-09-27)

A C enum or a range type is checked **wherever it comes in from C**, and
fails `out_of_range` for a number it doesn't allow: a call's return, what
C wrote through a `ptr.mut` you lent it (every element of the lent list,
checked when the call ends), and a `ptr.read`. Each is handled with
`otherwise` (`ptr.read['p'] otherwise Mode.emacs`).

**A C variable holds C's plain number** (owner, 2026-09-28: b): it is
read by its name, and reading a variable never fails anywhere in
Cyborg, so `extern.C 'm': Mode;` is refused. Declare it `i32` and
convert. **`to.i32[Mode.vi]`** turns a C enum back into its number,
which never fails but must fit the target, and that is how a C enum is
written into a plain C variable. Proposed alternative: `otherwise` after
a C variable's name (`'current' otherwise Mode.emacs`). The owner asked
for missed use cases; writing a C library's mode variable was the one,
and the conversion back answers it. Writing the
type is the request to check. To skip the check, write the plain type
(`i32`), and convert when you want the enum:

```
let $k := to.Key($code)-
otherwise:
    !case out_of_range:
        ret;
    !end of case out_of_range
end of otherwise;
```

`no_check` does not remove these. It turns off *check sites*, the bugs
("calling C here is fine"), not *failures*, which are answers you handle.
Returns were ruled so on 2026-09-22. A **callback's** parameter must be the
plain number: C calls it, so a failure would have nowhere to go.

Why it matters: a newer C library adds a case (`KEY_F13 = 302`) that
your binding doesn't list. Checked, that's an `out_of_range` your program
handles. Unchecked, it's undefined behaviour in a `match`. The check is a
comparison or two, under a nanosecond, beside a call into C.

Proposed and not chosen: `nocheck` vouching for the number where it is
already written (b); stopping the program on a bad number (c); a fourth
tier trusting C's answers too. The owner asked whether `no_check` was then
"lying" (it isn't, for the reason above), then chose this rule.
`examples/c_enum_convert.cy`.

## A range type as a C struct's field (owner, 2026-09-27: a)

```
%layout.C rec Rect:
    var w: int.sign.32[min: 0, max: 4096];
    var h: int.sign.32[min: 0, max: 4096];
end of %layout.C rec Rect
```

C sees the plain number, so the layout is unchanged (`Rect` is 8 bytes).
The range is Cyborg's promise: proven where Cyborg fills one in, and
checked at the door when a struct comes from C, failing `out_of_range`,
the ranged-return rule applied to fields. Proposed alternative: keep
refusing it, with plain fields checked by hand. A range on an `extern.C fn`
*parameter* is still refused; that was not asked.
`examples/c_struct_ranges.cy`.

## `layout.C enum` — C's enum (owner, 2026-09-27: a)

```
%layout.C enumeration Key:
    a = 65;
    b = 66;
    escape = 256;
end of enumeration Key
%external.C procedure IsKeyDown [$key: Key] -> boolean;
```

A C enum is a number with names, and here it is a Cyborg enum laid out
as C's: `match` covers every case, and it is owned like every enum
(2026-09-20). Its cases carry no data, and each says its number, as the C
header does. Cases are separated by `,`, as C's are and as every list in braces is
(owner, 2026-10-14: "Yea"); `incl.C` writes them so. Guessing the order is how bindings go wrong, so a missing
number is an error, and so are two cases with one number. It is 32 bits,
C's `int`. A number C hands back that matches no case fails `out_of_range`
at the door, like a ranged return: C can send any number. Only a
`layout.C enum` crosses to C. Proposed alternative: no enums at the
boundary, plain numbers with named constants, which loses the names in
types and the exhaustive `match`. Written numbers required, and 32 bits:
design claude's recommendation with a, not asked separately.
`examples/c_enum.cy`.

## `extern.C 'x': T` — a C global (decided 2026-09-21; `let` dropped 2026-09-25)

```
%extern.C let $lib_version: int.sign.32;    # C: extern const int lib_version;   never changes
%extern.C var $lib_verbose: int.sign.32;    # C: extern int lib_verbose;   may be written

!no_check let $v := $lib_version;       # reading C's variable is a no_check site
!no_check change $lib_verbose to 1;              # so is writing one
```

A C library's variables, named the way `extern.C fn` names its functions.
The type is required and C-shaped (numbers, `bool`, `ptr`, `layout.C`
structs); no value — C owns it; `mut` says it may be written, `!mut` is
meaningless. Every read and write is a `no_check` site, like an extern call:
nothing about C's memory can be proven. It is a plain value that copies
by itself — no verbs. Proposed alternatives: no extern data at all (a C
helper function per variable) and `extern.C 'x': ptr[T]` read through
`ptr.read`. Owner chose `extern let`, which lost its `let` with every
other declaration on 2026-09-25.

**Since 2026-10-27 a C variable says whether it can change**, as every declaration does (owner, 2026-10-27: "yes"): `let`
for one C declares `const`, which CyborgPL only reads, and `var` for every other, which CyborgPL may write. Not `const`:
CyborgPL's `const` is known while building, and a C constant's value is only there once the program links. The bare
form is refused.

**`errno` is not a plain global** on macOS and Linux — it is a macro
that calls a function — so `extern.C 'errno': i32;` would be wrong
there; the std kit offers `std::c.errno[]` backed by the real thing.
`extern.C 'x': T` is for genuinely plain C globals. `examples/extern_data.cy`.

## `layout.C` — a struct in C's shape

**A language's name is capitalized wherever CyborgPL writes it** (owner,
2026-10-02: "BIG"): `layout.C`, `inline.ASM`, `inline.MSL`, `inline.C`
when it comes. The owner said "inline.C has a big c" on 2026-10-01;
design claude generalized it then without recording it, and `layout.c`
(2026-09-19) kept its lowercase until this ruling. `layout.c` is refused,
naming `layout.C`.

**`inline.C` is in CyDT 1** (owner, 2026-10-24: "full C support comes in CyDT 1", then "Yes" to its including
`inline.C`): for function-like macros, `static inline` header functions and inlining small C across the boundary,
kayo compiling the block with Clang; it is CNC's (owner: "Yup"). Not designed yet.

```
%layout.C rec Timeval:
    var sec: int.sign.64;
    var usec: int.sign.64;
end of %layout.C rec Timeval
```

Only a `layout.C` struct may cross to C (in a signature or a `ptr[…]`); a
normal struct's layout is the prover's. Its fields must be C-shaped: numbers,
`bool`, `ptr[…]`, other `layout.C` structs — no `str`, no `list`.

## `size.of[T]`, `align.of[T]`

Compile-time constants, spelled like `to.`: `malloc[10 times size.of[Timeval]]`.
A `layout.C` struct has C's size; a normal struct's is whatever the prover
chose.

## Inline assembly — `inline.ASM` (decided 2026-09-20)

**The main names are `inline.ASM`, `inline.LL`, `inline.MSL`** (owner,
2026-10-07: "Main should now be `inline.ASM`, `inline.LL`, `inline.MSL`"),
replacing 2026-10-06's whole names; **the whole names and the older short
ones are MTM's** (owner: "`a`"): `inline.Assembly` and `inline.S` for
`inline.ASM`, `inline.LLVM_Intermediate_Representation` and `inline.LLVM`
for `inline.LL` (IEG's), `inline.Metal_Shading_Language` for `inline.MSL`
(the Metal limb's). `inline.S` came first as an alias (owner, 2026-10-02:
"Add `S` as an alias to `ASM`"), as assembly's `.S` files: the same
block, the same rules. Capitals, as every language's name: `inline.s` is
refused.

```
!no_check inline.ASM with [$fd -> x0, $buf -> x1, $len -> x2, $n <- x0, clobber x16]:
    mov x16, #4
    svc 0
end of inline.ASM
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
  copies the variable in before the block, `'v' <-- x0` stores the
  register into `'v'` after (declaring it, like `let 'v' := …`), `'v' <--> x0`
  both (`<-->` the main spelling since 2026-10-06; `<->`, `<<>>`, `<<<>>>`,
  `<---->`, `<--|-->`, `<-|->`, `<<|>>`, `<<<|>>>` the others); `clobber x16` says a register was overwritten.
- **A lend goes into a register as the address of the lent data**
  (owner, 2026-09-30: a): `lend 'text' --> x1`, and `lend.mut 'buf' --> x1`
  for data the assembly may write through — the rule an `extern` call
  already has, "a lend satisfies a pointer". Into a register only: a
  `<--` or `<-->` after a lend is refused. A plain pointer variable (`'buf'
  --> x1`, from: `malloc`) still works, for memory the program manages
  itself. The length, if the code needs one, is bound separately.
- **Compiler rule (2026-09-21):** on AArch64 the lowering spells `x29` as
  `fp` and `x30` as `lr` in what it hands LLVM — the ABI names. LLVM 22
  ignores a clobber of those two when written as `xN` (fixed in 23; Clang
  never hit it because it always emits the ABI names). The programmer
  keeps writing `x30`; the rename is invisible and harmless on 23.
- Always a `no_check` site. Per-architecture by nature.
- The oracle parses and checks all of this and then refuses to run it
  ("the oracle cannot run assembly"), with nine reject tests.

## C strings — `C.str.read` and `C.str.make` (owner, 2026-10-19)

A C string is bytes ending in a 0 byte; a `str.utf8` knows its own length. Two ways across, under `C.` beside
`C.int` (owner: "Isn't it `str`?"):

```
%extern.C proc puts [let $s: ptr[int.unsign.8]] -> int.sign.32;
let $name := %C.str.read($p)-
otherwise:
    case not_utf8:
        …
    end of case not_utf8
end of otherwise;              # a char* to its 0 byte → str.utf8
let $cs := %C.str.make(lend $name)-
otherwise:
    case has_zero_byte:
        …
    end of case has_zero_byte
end of otherwise;   # str.utf8 → a C string
puts(lend $cs);
```

- `C.str.read` fails `not_utf8` when the bytes are not UTF-8.
- `C.str.make` fails `has_zero_byte` when the text holds a 0 byte (C would cut it there), the check dropped where
  Precog proves there is none, a literal say (owner: a).
- A made C string's type is **`C.str`** (owner, 2026-10-19: "(a)"): `let 'cs': C.str = C.str.make[…]`.
- A bounded one says, always, whether its closing 0 counts (owner, 2026-10-19: "(a)"): `C.str[max: 255 bytes, exclude zero]`
  (text up to 255 bytes, the 0 on top, as `strlen` counts) or `C.str[max: 256 bytes, include zero]` (the whole buffer, as
  `char name[256]`). Bytes only. `C.str.make` into one fails `out_of_range` when the text does not fit (design claude's reading).
- A made C string is owned like any value (owner: a): freed when its owner's scope ends; `lend` it to C for a call,
  `give` it when C keeps it, and C frees it then. No `C.str.free`.
- Built in the oracle 2026-10-19: `examples/c_strings.cy`, reject tests `c_str_*`.

## What `C.` is for (owner, 2026-10-19)

`C.` is C's own vocabulary, what CyborgPL provides for talking to C: its types (`C.int`, `C.size_t`), the string
tools (`C.str.read`, `C.str.make`), the manifest's `C.libs`, and a raw C symbol after `=` (`= C.printf`). Names a
header brings into a file act as the file's own declarations, plain: functions (`printf[…]`), globals, and `#define`s,
which stay `'SQLITE_OK'` as since 2026-10-10 (the morning's `C.SQLITE_OK` was withdrawn the same day).

## A struct by value (owner, 2026-10-19)

A `layout.C struct` may be passed to C and returned from it by value, whatever its size:
`extern.C fn move_by ['p': Point, 'dx': f32] -> Point;`. kayo follows the platform's C rules (on Apple silicon, up to
16 bytes in registers, bigger ones copied with a hidden pointer, as C does), so it costs what the same call costs from
C. Already allowed before 2026-10-19 ("Only a `layout.C` struct may cross to C (in a signature …)"); the ruling only
said any size. The calling rules are kayo's; the oracle has nothing to stand in for.

## Variadic C functions, and a C function under another name (owner, 2026-10-19)

Both forms (owner: "What about both?"):

```
incl.%C.system["stdio.h"];                                        # printf comes with C's `...`
!no_check printf(lend $fmt, 5, lend $name);                     # the open form: any extras, on your word

%extern.C proc printf_count = %C.printf [let $format: ptr[int.unsign.8], let ...$n: int.sign.32] -> int.sign.32;
!no_check printf_count(lend $fmt, 5);                            # a fixed shape: its extras' types checked
```

- The open form is `...'args'` (owner, 2026-10-19: "(a)", as decided before), any extras, each call a `no_check` site.
- A fixed shape names and types its extras after the `...`, and those are checked; the call is still a `no_check` site,
  as every C call is (owner, 2026-10-19: "(a)"), since the format text can still disagree. It marks where the `...` starts, because on Apple
  silicon a variadic function's extras travel apart from its fixed arguments.
- `= C.printf` gives a C function another name in CyborgPL (owner: "(c)"); design claude's reading: any C function.
- Built in the oracle 2026-10-19, which stands in for `printf` (`%d %i %u %x %c %s %%`): `examples/c_strings.cy`,
  reject tests `c_rename_needs_c`, `c_fixed_shape_count`.

## Unions and bit-fields (owner, 2026-10-19)

```
%layout.C union Number:
    i: int.sign.32;
    f: float.bin.32;
end of union Number        # one of them at a time; C knows which
!no_check let $x := $n.i;                          # reading a field: on your word, listed in the build report
change $n.f to 1.5;                                 # writing: always safe

%layout.C rec Status:
    var mode: int.unsign.8[bits: 3];
    var ready: bool[bits: 1];
    var level: int.unsign.8[bits: 4];
end of %layout.C rec Status
change $s.mode to 5;                                # checked to fit: 3 bits hold 0 to 7
```

- A union's read is a `no_check` site (owner: a); a write needs nothing.
- A bit-field is `bits N` in the field type's limits bracket (owner: a; the placement is design claude's reading),
  laid out as clang does on macOS; reads always safe, writes checked to fit; only in `layout.C`.
- Built in the oracle 2026-10-19: `examples/c_union_bits.cy`, reject tests `union_*`, `bits_*`. Reading another field
  of a union gives its bytes as that field's type where the oracle can say (same-size numbers).

## Open

- (History: reading a C string was build-it-yourself until 2026-10-19; see "C strings" above.)
- Linking C libraries: `C.libs = [cyjet::C.lib[name = "sqlite3"]]` in `CyJet.cyr` (owner, 2026-10-19); where a library
  is looked for beyond the system's and Homebrew's folders, and a lone file's libraries, are not asked yet. Read in the
  oracle (it links nothing): `examples/c_libs_listed/`, reject test `c_lib_named_file`.
- A text's length is decided (`strings.md`: `str.bytes`, `str.points`,
  `str.graphemes`); this line is kept only to say so.
