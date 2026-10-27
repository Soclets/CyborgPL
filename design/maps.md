# Maps

> Revised 2026-10-07: code here is in the new syntax; an old spelling in the prose is history. The old → new table is `revision-2026-10-07.md`.

Decided 2026-09-22, one question at a time. The third collection, beside
`list` and `array` (`lists.md`): look a value up by a key.

```
var $phones: std::map[str.utf8, int.sign.32] = std::map.empty;
std::map.set(lend.mut $phones, key: "ann", value: 5551);                 # add, or replace an existing key's value
let $n := std::map.get(lend $phones, key: "ann") otherwise 0;    # a missing key is a failure
if std::map.has(lend $phones, key: "bob"):
    …
end of if
let $v := std::map.remove(lend.mut $phones, key: "bob") otherwise 0;   # hands the value back, like list.remove
let $count := std::map.count(lend $phones);                # u64
```

## A lookup lends its key (owner, 2026-10-25: "yes")

`map.set` keeps the key and value it is handed, so they come by `copy` or `give`. `map.get`, `map.has` and `map.remove`
only look at the key, so it may be lent:

```
let $n := std::map.get(lend $phones, key: lend $name) otherwise 0;   # no copy of the text, nothing kept
std::map.set(lend.mut $phones, key: copy $name, value: 5551);        # the map keeps this one
```

Until then every operation took `set`'s rule (design claude's, 2026-09-22, never asked), so a lookup copied its key.

## In the language (owner: a)

`map[K, V]` is a language type with type-headed operations — `map.empty`,
`map.length`, `map.has`, `map.get`, `map.set`, `map.remove` — like
`list.length`. the prover knows its shape natively (a length interval, a key
domain, a value domain) and the compiler owns the implementation: open
addressing, the kind that won the 2026-09-21 benchmark, never a linked
bucket per entry. Proposed alternatives: a std struct of lists (needs a
hash primitive or scans); Rust's shape, a std generic with compiler
hashing. Owner chose the language. `map` is a keyword.

## A missing key is a failure (owner: a)

`map.get` and `map.remove` are `--> V or fail[missing]`, so every lookup
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
  interface Key <T> { op == [$a: <T>, $b: <T>] --> bool;  fn hash [lent $v: <T>] --> u64; }
  ```

  compares and hashes by its own `op ==` and `fn hash` — a
  case-insensitive username, a normalised path. **Consistency is a check
  site** (owner: yes): "equal keys hash the same" must be proven by
  the prover for any type used as a key this way, or the final build
  refuses the map; `no_check` is the way out. This is the bug every other
  language's custom-hash map has and cannot catch; the oracle tests the
  rule on every comparison it makes.
- Not keys: functions, pointers, maps.

A map owns its keys and values: `set` takes a fresh value, a `copy`, or
a `give` — never a lend.

## A literal — `key = value` (owner: a)

```
let $phones := std::map["ann" = 5551, "bob" = 5552];
let $grid := std::map[[1, 2] = "tree", [3, 4] = "rock"];
let $one := std::map["cat" = 1,];
```

`key = value`, the spelling a struct construction uses for `field =
value`; keys are any expression, values any value. The empty map stays
`map.empty` (`map[]` is refused). A key written twice in one literal is
a compile error. Proposed alternatives: a list of two-element lists
(`map[["ann", 5551], …]`, more brackets); an arrow (`->` already means
"returns").

## A count bound, and bounds inside (owner: yes)

```
var $small: std::map[str.utf8[min: 1, max: 3 bytes], int.sign.32[min: 0, max: 10]][min: 0, max: 2] = std::map.empty;
```

`map[K, V][lo, hi]` bounds the number of entries, as `list[T][lo, hi]`
bounds a list: checked after every `set` and `remove`, and
`map.length` of it is a `u64[lo, hi]`. The key and value types are
ordinary types, so their own bounds compose: the key's text length and
the value's range above are checked on `set`. Nothing new — the pieces
fitting together (owner asked "what about the things inside?").

## Walking a map — say which, in insertion order (owner: c, a)

```
for $k in std::map.keys(lend $phones):
    …
end of for
for $v in std::map.values(lend $phones):
    …
end of for
for $e in std::map.pairs(lend $phones):
    … $e.key … $e.value …
end of for
```

A map has no single walk, so the walk says which — like `str.each.point`
says its unit. Proposed alternatives: the key alone; the pair alone.

The order is **insertion order**: entries come back as they were `set`
(a replaced key keeps its place). Measured 2026-09-22 (Rust `HashMap`
vs `IndexMap`, 5M `i64` entries): insertion-ordered is 13% slower on
lookups and 11 bytes/entry larger, slightly faster on inserts, and 13×
faster to walk. Proposed alternatives: an unspecified-but-deterministic
order (unspecified output punches a hole in differential testing — the
oracle and the compiler would be allowed to disagree); sorted by key
(a sort per walk; structs have no `<`).

## The variant is the prover's to pick, or the writer's (owner: yes, a)

The order costs only the list that serves the walks. the prover sees the
whole program: a map that is never walked gets the unordered
representation — the 13% and the bytes back, automatically — the same
trick as `list` vs `array`. And the writer may say, by the dot rule:

- `map[K, V]` — the prover decides.
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
carry 8 bytes for a list it does not have). the prover already knows which
a given map is, since it chose the variant; a `map.unordered` written
by hand is always the small one.

## Open

- the prover's key and value domains in detail (`precog.md`).
