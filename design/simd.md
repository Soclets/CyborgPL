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
        change 'acc' = 'acc' or.bit simd.load[lend 'bytes', pos 'i'];
    }
    return simd.max['acc'] < 128;
}
```

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
in `Synthia.cplm`, never from the host.

## The operators, element-wise (owner: a)

Every operator a plain `T` has works on `simd[T, N]`, lane by lane —
`+ - x / mod`, `and.bit or.bit not.bit xor shl shr`, the wrapping
forms. `'a' or.bit 'b'` is the same bit-for-bit OR as on one number,
done N times. Proposed alternative: named operations only
(`simd.or['a', 'b']`) — nothing new in the grammar, worse to read
everywhere.

**Comparisons give a vector of answers**: `'a' < 'b'` on
`simd[int32, 4]` is a `simd[bool, 4]`, one answer per lane. A vector
is never a single `bool`, so `if 'a' < 'b' { … }` is a compile error
that names the fold to use.

## The operations

- `'v'[pos 1]` — one lane, 1-based like every index; a check site.
- `simd.fill[7]` — every lane the same value.
- `simd.load[lend 'xs', pos 'i']` — N elements of an `array` or `list`
  into a vector; a check site (is `'i' + N - 1` inside?).
- `simd.store[lend.mut 'xs', pos 'i', 'v']` — the other way.
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
end, a bad element type, and a position without `pos`. Two things the
oracle had to pick and this document has not said: a scalar broadcasts
against a vector (`'v' + 1`), and `simd.sum` is a check site while
`max`/`min` are not.

## Open

- Which `T` beyond the integers and bins: `bool` exists as a
  comparison's result; is `simd[bool, N]` writable directly?
- Shuffles and lane-crossing operations (reverse, interleave, a
  general permute) — none designed; the folds cover the common cases.
- Masked load/store (fill only some lanes), which is what a tail
  shorter than N needs; today the tail is a plain loop.
- Whether `simd` values may be struct fields or list elements.
