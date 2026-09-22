# Maps

Decided 2026-09-22, one question at a time. The third collection, beside
`list` and `array` (`lists.md`): look a value up by a key.

```
let mut 'phones': map[str.utf8, int32] = map.empty;
map.set[lend.mut 'phones', "ann", 5551];                 # add, or replace an existing key's value
let 'n' := map.get[lend 'phones', "ann"] otherwise 0;    # a missing key is a failure
if map.has[lend 'phones', "bob"] { … }
let 'v' := map.remove[lend.mut 'phones', "bob"] otherwise 0;   # hands the value back, like list.remove
let 'count' := map.length[lend 'phones'];                # uint64
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

## Open

- Walking a map (`for` over its keys, its pairs?) and in what order —
  the oracle keeps insertion order; the language has not said.
- A literal map (`map[…]` with pairs written out) — today `map.empty` +
  `set`.
- A length bound (`map[K, V][0, 100]`), by analogy with lists.
- Precog's key and value domains in detail (`precog.md`).
