# Maps

Decided 2026-09-22, one question at a time. The third collection, beside
`list` and `array` (`lists.md`): look a value up by a key.

```
mut 'phones': map[str.utf8, int32] = map.empty;
map.set[lend.mut 'phones', "ann", 5551];                 # add, or replace an existing key's value
'n' := map.get[lend 'phones', "ann"] otherwise 0;    # a missing key is a failure
if map.has[lend 'phones', "bob"] { … }
'v' := map.remove[lend.mut 'phones', "bob"] otherwise 0;   # hands the value back, like list.remove
'count' := map.length[lend 'phones'];                # uint64
```

## In the language (owner: a)

`map[K, V]` is a language type with type-headed operations — `map.empty`,
`map.length`, `map.has`, `map.get`, `map.set`, `map.remove` — like
`list.length`. Precog knows its shape natively (a length interval, a key
domain, a value domain) and the compiler owns the implementation: open
addressing, the kind that won the 2026-09-21 benchmark, never a linked
bucket per entry. Proposed alternatives: a std struct of lists (needs a
hash primitive or scans); Rust's shape, a std generic with compiler
hashing. Owner chose the language. `map` is a keyword.

## A missing key is a failure (owner: a)

`map.get` and `map.remove` are `-> V or fail[missing]`, so every lookup
says what happens: `otherwise 0`, `otherwise fail`, `otherwise { missing
{ … } }`. A bad index is a bug and a check site; a missing key is an
ordinary Tuesday, and the language already had the word for that. `has`
is a bool for when the value is not wanted. Proposed alternatives: a
check site guarded by `has`; a found/missing enum to `match`.

## What can be a key (owner: c)

- Plain values, text, chars, and range types of them — compared and
  hashed by value.
- **Structs, enums, and lists of keys — by structure**: the compiler
  derives comparison and hashing field by field, automatically, the way
  interfaces are satisfied by shape. `map[Point, str.utf8]` just works.
- **A type's own rule**: a struct or enum that satisfies

  ```
  interface Key { op == ['a': <T>, 'b': <T>] -> bool;  fn hash [lent 'v': <T>] -> uint64; }
  ```

  compares and hashes by its own `op ==` and `fn hash` — a
  case-insensitive username, a normalised path. **Consistency is a check
  site** (owner: yes): "equal keys hash the same" must be proven by
  Precog for any type used as a key this way, or the final build
  refuses the map; `nocheck` is the way out. This is the bug every other
  language's custom-hash map has and cannot catch; the oracle tests the
  rule on every comparison it makes.
- Not keys: functions, pointers, maps.

A map owns its keys and values: `set` takes a fresh value, a `copy`, or
a `give` — never a lend.

## A literal — `key = value` (owner: a)

```
'phones' := map["ann" = 5551, "bob" = 5552];
'grid' := map[[1, 2] = "tree", [3, 4] = "rock"];
'one' := map["cat" = 1,];
```

`key = value`, the spelling a struct construction uses for `field =
value`; keys are any expression, values any value. The empty map stays
`map.empty` (`map[]` is refused). A key written twice in one literal is
a compile error. Proposed alternatives: a list of two-element lists
(`map[["ann", 5551], …]`, more brackets); an arrow (`->` already means
"returns").

## A count bound, and bounds inside (owner: yes)

```
mut 'small': map[str.utf8[1, 3 bytes], int32[0, 10]][0, 2] = map.empty;
```

`map[K, V][lo, hi]` bounds the number of entries, as `list[T][lo, hi]`
bounds a list: checked after every `set` and `remove`, and
`map.length` of it is a `uint64[lo, hi]`. The key and value types are
ordinary types, so their own bounds compose: the key's text length and
the value's range above are checked on `set`. Nothing new — the pieces
fitting together (owner asked "what about the things inside?").

## Walking a map — say which, in insertion order (owner: c, a)

```
for 'k' in map.keys[lend 'phones'] { … }
for 'v' in map.values[lend 'phones'] { … }
for 'e' in map.pairs[lend 'phones'] { … 'e'.key … 'e'.value … }
```

A map has no single walk, so the walk says which — like `str.each.point`
says its unit. Proposed alternatives: the key alone; the pair alone.

The order is **insertion order**: entries come back as they were `set`
(a replaced key keeps its place). Measured 2026-09-22 (Rust `HashMap`
vs `IndexMap`, 5M `int64` entries): insertion-ordered is 13% slower on
lookups and 11 bytes/entry larger, slightly faster on inserts, and 13×
faster to walk. Proposed alternatives: an unspecified-but-deterministic
order (unspecified output punches a hole in differential testing — the
oracle and the compiler would be allowed to disagree); sorted by key
(a sort per walk; structs have no `<`).

## The variant is Precog's to pick, or the writer's (owner: yes, a)

The order costs only the list that serves the walks. Precog sees the
whole program: a map that is never walked gets the unordered
representation — the 13% and the bytes back, automatically — the same
trick as `list` vs `array`. And the writer may say, by the dot rule:

- `map[K, V]` — Precog decides.
- `map.ordered[K, V]` — always insertion-ordered, walkable.
- `map.unordered[K, V]` — never keeps order and **cannot be walked**:
  `map.keys` on it is a compile error. Choosing it *is* the promise, so
  no unspecified order ever reaches a program.

Proposed alternative: two words like `list`/`array`; there is no honest
second noun. A sorted map (Rust's `BTreeMap`) would be a genuinely
different type, for later if wanted.

## Layout (owner, 2026-09-22)

One flat table, open addressing (never a linked bucket per entry). An
**unordered map's handle is three fields** — a pointer to the table, a
count, a capacity (24 bytes) — and an **ordered map's is four**, the
fourth pointing at the insertion-order list. Two shapes, not one with a
null pointer (proposed, and declined: a map that is never walked would
carry 8 bytes for a list it does not have). Precog already knows which
a given map is, since it chose the variant; a `map.unordered` written
by hand is always the small one.

## Open

- Precog's key and value domains in detail (`precog.md`).
