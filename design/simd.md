# SIMD — `simd[T, N]`

Decided 2026-09-22, one question at a time. A CPU has registers that
hold several numbers side by side and instructions that work on all of
them at once. A plain `for` over an `array` should reach the same
instructions by itself — the prover proves the checks away, LLVM's
vectoriser does the rest — and that is the path to take first. These
types are for when it does not, or when the algorithm is a vector
algorithm.

```
fn is_ascii [lent $bytes: arr[u8]] -> bool:
    var $acc = simd.fill(value: 0);                          # simd[u8, count 16]
    for $i [from: 1, to: arr.len(lend $bytes), by: 16]:
        change $acc to $acc or.bit simd.load(lend $bytes, pos: $i);
    ;for
    ret simd.max($acc) < 128;
;fn is_ascii
```


**Lanes are checked at every operation, at the lane's type** (owner,
2026-09-29: b) — not exact in between as plain numbers are (2026-09-23),
since wider lanes would halve what a register holds. `('a' plus 'a') minus 'a'`
on `simd[u8, count: 16]` of 200s is a check site at the `+`. The wrapping
forms (`+~` …) wrap lane by lane and never check. For exact maths in
between, use a plain loop.

## The spelling — `simd[T, N]` (owner)

The language's bracket rule for type parameters, as `list[i32]` and
`map[K, V]`: `simd[u8, count: 16]`, `simd[f32, count: 4]`. Proposed
alternatives: `uint8x16` (Swift's and Rust's look — a naming
convention the parser must know), `u8[16]` (collides with the
range-type shape `i32[min: 0, max: 100]`).

## Only the target's own shapes (owner: b)

`simd[T, N]` is legal only where the target has that register. On
ARM64 (128-bit NEON): `simd[u8, count: 16]`, `simd[i32, count: 4]`,
`simd[f32, count: 4]`, `simd[f64, count: 2]`, and the like. `simd[u8, count: 64]` is
a compile error naming what the target does have. Proposed
alternative: any power-of-two N, the compiler splitting or padding —
owner: "shouldn't they just use the normal way?" A program reaching for
`simd` is writing for a machine, and a type that pretends otherwise
lies; portability is the plain loop. A `simd` program built for another
target fails to build rather than running slowly — the honesty
`inline.ASM` already has. The shapes come from the target description
in the manifest (`Synthia.ttsr` / `CyJet.cyr`), never from the host.

## The operators, element-wise (owner: a)

Every operator a plain `T` has works on `simd[T, N]`, lane by lane —
`plus minus times`, `div.X` and `mod.X`, `and.bit or.bit not.bit xor shl shr`, the wrapping
forms. `'a' or.bit 'b'` is the same bit-for-bit OR as on one number,
done N times. Proposed alternative: named operations only
(`simd.or['a', 'b']`) — nothing new in the grammar, worse to read
everywhere.

**Comparisons give a vector of answers**: `'a' < 'b'` on
`simd[i32, count: 4]` is a `simd[bool, count: 4]`, one answer per lane. A vector
is never a single `bool`, so `if 'a' < 'b' { … }` is a compile error
that names the fold to use.

## The operations

- `'v'[1]` — one lane, 1-based like every index; a check site.
- `change 'v'[3] = 30;` — one lane written, like a list item; a position
  outside the vector is a check site (owner, 2026-09-29: a). One
  instruction (`INS` on ARM).
- `simd.fill[value: 7]` — every lane the same value. The lanes are what the declared type says, and the value is
  fitted into them as a typed `let` fits it (owner, 2026-10-19): `let 'r': simd[i32, count: 4] = simd.fill[value: 'b'];`
  with a `u8` 'b' gives four `i32` lanes; a value that does not fit is refused.
- `simd.load[lend 'xs', 'i']` — N elements of an `array` or `list`
  into a vector; a check site (is `'i' plus N minus 1` inside?).
- `simd.store[lend.mut 'xs', 'i', value: 'v']` — the other way.
- `simd.all['m']` / `simd.any['m']` — a `simd[bool, N]` to one `bool`.
- `simd.max['v']` / `simd.min['v']` / `simd.sum['v']` — a numeric
  vector to one value. `sum` is a plain number, worked out in full and
  checked where it lands (owner, 2026-10-07); `simd.sum.lane['v']` checks it
  at the lane's type, and `simd.sum.wrap['v']` wraps it there as the chip's
  own sum does, never a check (owner, 2026-10-07: "There should be a way to
  do the other option too", then "yea"). Until 2026-10-07 this said "`sum` is
  a check site (it can overflow `T`)".

## What the prover knows

A lane is a plain value, so a vector carries **a domain per lane** and
every operator's rule is the scalar rule applied lane-wise. A load
whose index is proven inside the array needs no check; `simd.sum` whose
lanes are proven small needs no overflow check. Nothing about SIMD is
special to the prover — it is N of what it already does.

## In the oracle (2026-09-22)

All of the above, with the oracle's target fixed as ARM64's 128 bits.
`examples/simd.cy` is the `is_ascii` program plus arithmetic,
comparisons, folds and a store; five reject tests cover a shape the
target does not have, a vector used as a condition, a load past the
end, and a bad element type. Two things the
oracle had to pick and this document has not said: a scalar broadcasts
against a vector (`'v' plus 1`), and `simd.sum` is a check site while
`max`/`min` are not.

## A mask is a type: `simd[bool, N]` (owner, 2026-09-27: a)

```
fn all_on [let $m: simd[bool, count: 4]] -> bool:
    ret simd.all($m);
;fn all_on
let $m: simd[bool, count: 4] := $a > simd.fill(value: 10);
let $flip := not $m;
```

A vector comparison's result can be written as a type, so a function can
take a mask and a declaration can name one. It has as many lanes as the
vectors it compares: 2, 4, 8 or 16 on a 128-bit target. `simd.fill[true]`,
`simd.all`, `simd.any` apply, and `not`, `and`, `or` work lane by lane.
Proposed alternative: a mask exists only as a comparison's result, used
on the spot. `examples/simd_mask.cy`.

## The leftover end: `simd.load.some` / `simd.store.some` (owner, 2026-09-27: a)

```
let $v: simd[i32, count: 4] := simd.load.some(lend $xs, pos: 9, count: 2);   # 2 items; lanes 3–4 are zero
simd.store.some(lend.mut $xs, pos: 9, count: 2, value: $v);               # writes lanes 1–2 only
```

Every vector loop over a list whose length is not a multiple of N has a
tail. With these the tail is one more trip of the same loop, not a second
scalar loop. k is 0 to N, and the k items must lie in the list, a check
site the prover proves like `simd.load`'s. The other lanes of a load are the
element type's zero. Newer hardware does this directly (SVE predicates,
AVX-512 masks), and elsewhere the compiler lowers it. Proposed
alternative: leave the tail a plain loop. The names `load.some` /
`store.some` were design claude's proposal, flagged as placeholders, and
went through with a. `examples/simd_tail.cy`.

## A simd value goes anywhere a number does (owner, 2026-09-27: a)

```
rec Particle:
    var place: simd[f32, count: 4];
    var vel: simd[f32, count: 4];
;rec Particle
let $history: std::ls[simd[f32, count: 4]] := [];
```

Struct fields, list elements, parameters and returns. It is a plain,
fixed-size value that copies by itself, like a number, and the compiler
aligns it to the register width (16 bytes on a 128-bit target). Graphics
and physics keep positions this way. Proposed alternative: SIMD values
local only, with plain numbers stored and reloaded each time.
`examples/simd_fields.cy`.

## Rearranging lanes: `simd.shuffle` (owner, 2026-09-27: a)

```
let $r := simd.shuffle($v, [4, 3, 2, 1]);      # reversed
let $p := simd.shuffle($v, [2, 1, 4, 3]);      # neighbours swapped
let $b := simd.shuffle($v, [1, 1, 1, 1]);      # lane 1 everywhere
```

One operation for every rearrangement: the vector, then its new order as
lane numbers (1-based, as positions are), one per lane, and repeats are
allowed. The order must be fixed numbers known when compiling, so it is
one instruction on every chip and the prover sees through it. A lane that
does not exist is an error. Proposed alternative: nothing yet, since the
folds cover the common cases. Shuffling two vectors together
(interleave) was not asked. `examples/simd_shuffle.cy`.

## Open

**Lanes are a standard width** (owner, 2026-10-03: a): a `!std.` integer
(`!std.uint4`, `!std.int256`) is never a lane, since no chip has one and
an emulated vector would be slower than plain code.
