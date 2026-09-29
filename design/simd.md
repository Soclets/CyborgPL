# SIMD — `simd[T, N]`

Decided 2026-09-22, one question at a time. A CPU has registers that
hold several numbers side by side and instructions that work on all of
them at once. A plain `for` over an `array` should reach the same
instructions by itself — Precog proves the checks away, LLVM's
vectoriser does the rest — and that is the path to take first. These
types are for when it does not, or when the algorithm is a vector
algorithm.

```
fn is_ascii [lent 'bytes': array[uint8]] -> bool {
    mut 'acc' := simd.fill[0];                          # simd[uint8, 16]
    for 'i' in range[1, array.length[lend 'bytes'], 16] {
        change 'acc' = 'acc' or.bit simd.load[lend 'bytes', 'i'];
    }
    return simd.max['acc'] < 128;
}
```


**Lanes are checked at every operation, at the lane's type** (owner,
2026-09-29: b) — not exact in between as plain numbers are (2026-09-23),
since wider lanes would halve what a register holds. `('a' + 'a') - 'a'`
on `simd[uint8, 16]` of 200s is a check site at the `+`. The wrapping
forms (`+~` …) wrap lane by lane and never check. For exact maths in
between, use a plain loop.

## The spelling — `simd[T, N]` (owner)

The language's bracket rule for type parameters, as `list[int32]` and
`map[K, V]`: `simd[uint8, 16]`, `simd[bin32, 4]`. Proposed
alternatives: `uint8x16` (Swift's and Rust's look — a naming
convention the parser must know), `uint8[16]` (collides with the
range-type shape `int32[0, 100]`).

## Only the target's own shapes (owner: b)

`simd[T, N]` is legal only where the target has that register. On
ARM64 (128-bit NEON): `simd[uint8, 16]`, `simd[int32, 4]`,
`simd[bin32, 4]`, `simd[bin64, 2]`, and the like. `simd[uint8, 64]` is
a compile error naming what the target does have. Proposed
alternative: any power-of-two N, the compiler splitting or padding —
owner: "shouldn't they just use the normal way?" A program reaching for
`simd` is writing for a machine, and a type that pretends otherwise
lies; portability is the plain loop. A `simd` program built for another
target fails to build rather than running slowly — the honesty
`inline.ASM` already has. The shapes come from the target description
in the manifest (`Synthia.ttsr` / `CCA.cplr`), never from the host.

## The operators, element-wise (owner: a)

Every operator a plain `T` has works on `simd[T, N]`, lane by lane —
`+ - x div mod`, `and.bit or.bit not.bit xor shl shr`, the wrapping
forms. `'a' or.bit 'b'` is the same bit-for-bit OR as on one number,
done N times. Proposed alternative: named operations only
(`simd.or['a', 'b']`) — nothing new in the grammar, worse to read
everywhere.

**Comparisons give a vector of answers**: `'a' < 'b'` on
`simd[int32, 4]` is a `simd[bool, 4]`, one answer per lane. A vector
is never a single `bool`, so `if 'a' < 'b' { … }` is a compile error
that names the fold to use.

## The operations

- `'v'[1]` — one lane, 1-based like every index; a check site.
- `change 'v'[3] = 30;` — one lane written, like a list item; a position
  outside the vector is a check site (owner, 2026-09-29: a). One
  instruction (`INS` on ARM).
- `simd.fill[7]` — every lane the same value.
- `simd.load[lend 'xs', 'i']` — N elements of an `array` or `list`
  into a vector; a check site (is `'i' + N - 1` inside?).
- `simd.store[lend.mut 'xs', 'i', 'v']` — the other way.
- `simd.all['m']` / `simd.any['m']` — a `simd[bool, N]` to one `bool`.
- `simd.max['v']` / `simd.min['v']` / `simd.sum['v']` — a numeric
  vector to one value. `sum` is a check site (it can overflow `T`).

## What Precog knows

A lane is a plain value, so a vector carries **a domain per lane** and
every operator's rule is the scalar rule applied lane-wise. A load
whose index is proven inside the array needs no check; `simd.sum` whose
lanes are proven small needs no overflow check. Nothing about SIMD is
special to Precog — it is N of what it already does.

## In the oracle (2026-09-22)

All of the above, with the oracle's target fixed as ARM64's 128 bits.
`examples/simd.cpls` is the `is_ascii` program plus arithmetic,
comparisons, folds and a store; five reject tests cover a shape the
target does not have, a vector used as a condition, a load past the
end, and a bad element type. Two things the
oracle had to pick and this document has not said: a scalar broadcasts
against a vector (`'v' + 1`), and `simd.sum` is a check site while
`max`/`min` are not.

## A mask is a type: `simd[bool, N]` (owner, 2026-09-27: a)

```
fn all_on ['m': simd[bool, 4]] -> bool { return simd.all['m']; }
'm': simd[bool, 4] = 'a' > simd.fill[10];
'flip' := not 'm';
```

A vector comparison's result can be written as a type, so a function can
take a mask and a declaration can name one. It has as many lanes as the
vectors it compares: 2, 4, 8 or 16 on a 128-bit target. `simd.fill[true]`,
`simd.all`, `simd.any` apply, and `not`, `and`, `or` work lane by lane.
Proposed alternative: a mask exists only as a comparison's result, used
on the spot. `examples/simd_mask.cpls`.

## The leftover end: `simd.load.some` / `simd.store.some` (owner, 2026-09-27: a)

```
'v': simd[int32, 4] = simd.load.some[lend 'xs', 9, 2];   # 2 items; lanes 3–4 are zero
simd.store.some[lend.mut 'xs', 9, 2, 'v'];               # writes lanes 1–2 only
```

Every vector loop over a list whose length is not a multiple of N has a
tail. With these the tail is one more trip of the same loop, not a second
scalar loop. k is 0 to N, and the k items must lie in the list, a check
site Precog proves like `simd.load`'s. The other lanes of a load are the
element type's zero. Newer hardware does this directly (SVE predicates,
AVX-512 masks), and elsewhere the compiler lowers it. Proposed
alternative: leave the tail a plain loop. The names `load.some` /
`store.some` were design claude's proposal, flagged as placeholders, and
went through with a. `examples/simd_tail.cpls`.

## A simd value goes anywhere a number does (owner, 2026-09-27: a)

```
struct Particle { place: simd[bin32, 4]  vel: simd[bin32, 4] }
'history': list[simd[bin32, 4]] = [];
```

Struct fields, list elements, parameters and returns. It is a plain,
fixed-size value that copies by itself, like a number, and the compiler
aligns it to the register width (16 bytes on a 128-bit target). Graphics
and physics keep positions this way. Proposed alternative: SIMD values
local only, with plain numbers stored and reloaded each time.
`examples/simd_fields.cpls`.

## Rearranging lanes: `simd.shuffle` (owner, 2026-09-27: a)

```
'r' := simd.shuffle['v', [4, 3, 2, 1]];      # reversed
'p' := simd.shuffle['v', [2, 1, 4, 3]];      # neighbours swapped
'b' := simd.shuffle['v', [1, 1, 1, 1]];      # lane 1 everywhere
```

One operation for every rearrangement: the vector, then its new order as
lane numbers (1-based, as positions are), one per lane, and repeats are
allowed. The order must be fixed numbers known when compiling, so it is
one instruction on every chip and Precog sees through it. A lane that
does not exist is an error. Proposed alternative: nothing yet, since the
folds cover the common cases. Shuffling two vectors together
(interleave) was not asked. `examples/simd_shuffle.cpls`.

## Open

