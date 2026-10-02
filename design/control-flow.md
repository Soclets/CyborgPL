# Control flow

Decided 2026-09-14; spelling revised 2026-09-19 (owner): bare keywords,
conditions without `[ ]`, one `for`.

## Branching

```
if 'x' > 10 {
    …
} else if 'x' > 5 {
    …
} else {
    …
}
```

- The condition is a bare expression followed by `{`. It must be a `bool`.
  No truthiness: `if 'n' { }` on an `int32` is a compile error.
- `else if` — two bare words (was `else.if` until 2026-09-19).
- Braces are mandatory, even around one statement. Because construction
  uses `[`, not `{`, a struct value in a condition is never ambiguous.
- Precog narrows the range of every variable inside a branch by its
  condition; this is where guardrails live.

## Scopes

A block is a scope. A variable declared in it is gone when it ends.
**Shadowing is never silent** (decided 2026-09-14): a name visible from an
enclosing scope cannot be redeclared with `:=` or `=`. It *can* be
deliberately hidden with `$:=` / `$=`, the `$` marking the intent:

```
let 'x' := 1;
if true {
    let 'x' $:= 2;            # shadows the outer 'x' until the block ends
    let 'y': int32 $= 3;      # error: there is no 'y' to shadow
}
```

A name marked `!$` cannot be shadowed, and neither can a loop's
counter. An `immut` one can (2026-09-25): shadowing makes a new variable
and changes nothing, so `immut` only forbids `change`.

## Loops

Three bare keywords (revised 2026-09-19; until then `loop`, `loop.while`,
`loop.temp.for`, `loop.temp.range`):

```
loop {                                         # forever, until break
    …
}

while 'count' < 100 {                          # while a condition holds
    …
}

for 'item' in lend 'items' {                   # each element; 'item' gone after
    …
}

for 'i' in range[0, 1000] {                    # 'i' from 0 to 1000 inclusive
    …
}
```

### `temp` (default) / `perm` — before the name

`temp`: the variable exists only inside the loop — **the default** (decided
2026-09-19; until then stating one of the two was mandatory). Writing `temp`
is allowed and changes nothing, like `mut`. `perm`: it is still there
afterwards, holding **whatever the loop last gave it** — the last value the
body ran with after a full run (the stop value when the step lands on it; for
`range[0, 10, 3]` it is 9), the value at a `break`, the start value if the
body never ran. One rule for every way a loop can end.

```
for perm 'i' in range[0, 100] {
    if found['i'] { break; }
}
std::print.useful.stdout["stopped at ", 'i', \n];
```

`perm` on a list loop keeps the last element visited (an empty list is an
error: there is nothing to keep).

### `range[…]`

One `for`; the range is a value it runs over (decided 2026-09-19, merging
the old `loop.range` kind). `range[start, stop]` — **inclusive**: "start
from, stop at"; the writer sees `'i'` reach `stop`. A third slot is the
**step**: `range[0, 100, 10]`; negative counts down: `range[10, 0, -1]`;
a step of 0 is a compile error. Start past stop with a positive step runs
zero times.

The counter is declared like a variable: bare name → best type from the
bounds; `'i': int32` → explicit, every value must fit. **The step is not
one of those values** (owner, 2026-09-23): it is its own signed number,
an instruction to go up or down, so `for 'i': uint64 in range[3, 1, -1]`
is legal — `'i'` is 3, 2, 1, every one a fine `uint64`. Counting down a
list by position needs this, since a position is a `uint64`. Proposed
alternative: the step must fit the counter's type too, which would make
a backwards walk impossible to write without a signed counter and a
conversion at every index. **The compiler compares before it steps**:
`range[3, 0, -1]` ends at 0, and computing "next = 0 − 1" to test would
underflow a `uint64` — an implementation rule the difference in
behaviour would otherwise hide. The bounds carry no
`:=`/`=` markers (they would say the same thing twice). In a `#strict`
file each bound is labelled and the step always written,
`range[from 2, to 8, by 2]` (owner, 2026-09-27; `core-features.md`):

```
for 'i' in range[0, 9] { }                 # like  'i' := 0;
for 'i': int8 in range[0, 9] { }           # like  'i': int8 = 0;
```

**`range[…]` stays part of the `for`** (owner, 2026-09-23): it is not a
value, cannot be stored, passed or returned. The four things a
first-class range is used for elsewhere are answered otherwise here —
slicing by `list.slice` / `std::str.slice` with position arguments like
every other operation; bounds passed to a function as two parameters or
a three-line `struct Span { from: int32  to: int32 }` the writer owns;
looping twice by writing the loop twice; and chaining, which needs
iterators the language does not have. Nothing had asked for it since it
was opened on 2026-09-19.

Historic note: `range[…]` is only decided as what a `for` runs over; as a standalone value
it is undecided and the oracle refuses it.

**The body cannot assign to the counter** (or to a list loop's element): it
belongs to the loop, so the header alone says how many times the body runs
and what `'i'` is on every pass — which is what keeps `range` loops easy for
Precog. Skipping ahead is a `while` with your own counter.

### Modifier order

`for [!c] [seq|para] [temp|perm] 'name'[: type] in …` — `!c` (no cheating,
2026-10-01) first, then `seq` or `para` (opt out of or force parallelism,
see `parallelism.md`), then `perm` if the variable outlives the loop, then
the name, reading "for, sequentially, a permanent 'i'": `for seq perm 'i':
int32 in range[0, 9] { }`. The strongest modifier comes first. Plain
`for 'i' in …` is the common case.

### Not in the language

Labelled `break` (leave an outer loop from an inner one: use a flag, or a
`perm` counter) and character ranges — both considered and skipped,
2026-09-17. Index-alongside-element in `for` is parked.

- `break;` and `continue;` as usual.
- How each element of a `for` arrives is the ownership question with the
  same verbs: `lend 'items'` lends each element; `give 'items'` consumes the
  list. A list loop's variable takes no type: the element's type is the
  list's.
- For Precog: `range` and a list loop have a bound for free; `loop` and
  `while` are where invariant inference or a programmer-supplied bound is
  needed for `check` to succeed.
