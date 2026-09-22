# Concurrency — `concur`, channels, `whichever`

Decided 2026-09-22, one question at a time. Not parallelism
(`parallelism.md`: the same work on many pieces, Precog spreads a `for`
over cores or a GPU, the programmer writes nothing). Concurrency is
*different* things that wait on the outside world — ten connections, a
download beside a keyboard — where the waiting is the point.

```
let 'results' := channel[str.utf8];
let 'a' := "a.com";
concur with [lend 'results', give 'a'] { channel.send[lend 'results', fetch[lend 'a']]; }   # a call's result is fresh: no verb
concur with [lend 'results'] { channel.send[lend 'results', fetch["b.com"]]; }
for 'i' in range[1, 2] {
    let 'page' := channel.take[lend 'results'] otherwise "";    # waits; whichever finished first
    std::printu.stdout[str.bytes[lend 'page'], \n];
}
```

## The model — tasks and channels (owner: b)

A word starts a task; tasks talk through channels; a value sent is
`give`n, so no two tasks ever hold one thing — no locks, because nothing
is shared, and Precog checks that nothing is. Ownership does the work
the language already paid for. Proposed alternatives: Precog overlaps
waiting calls automatically (would rarely fire — real concurrent
programs are about shared state); `async`/`await` (colours every
caller).

## `concur` (owner, over `task`)

```
concur { … }                       # a block, run alongside
concur fetch[lend 'url'];          # a call, run alongside
```

A bare word in front of a statement, the family of `check` / `nocheck`.
The first six letters of *concurrent*; the owner weighed the English
reading ("agree") against `task` and chose `concur`.

## Captures — explicit, all verbs (owner: a)

A task can outlive the code around it, so it names what it takes, as an
anonymous function does (`functions-as-values.md`): `with [give 'url',
copy 'n', lend 'results']`. `give` moves it in; `copy` duplicates;
**`lend` is a check site** — Precog proves the task ends before the
lender's scope does (structured tasks make that provable; see below);
unproven, the tiers. Proposed alternative: `give` and `copy` only.

## Structured, with `concur.detached` (owner: c)

A task lives inside the block that started it: when a `{ }` ends, it
waits for every `concur` it started. A task cannot outlive its starter,
so a lend into it is provable, MAIN ending means everything finished,
and a failure has a place to land. **`concur.detached { … }`** is the
escape for work whose lifetime is the program's, not its starter's — an
autosave loop started from a short `setup` function, a log flusher, a
library's listener — and it may capture only `give` / `copy`, since no
scope protects it. A block that fails cancels the tasks it still has
running. Proposed alternatives: structured only; detached only (Go —
the goroutine leak).

## A task's failure — `otherwise` on the `concur` (owner: a)

```
concur with [give 'url'] { let 'p' := fetch['url'] otherwise fail; … }
    otherwise { no_network { std::printu.stdout["offline", \n]; } };
```

A task that can fail says so where it is written, like every failing
call; the handler runs **when the enclosing block ends**, the point at
which the task's result is known. Proposed alternatives: `otherwise` on
the block; tasks cannot fail (plumb it through a channel by hand).

## Channels

```
let 'c' := channel[str.utf8];                    # hand-off: send waits for a taker
let 'q' := channel[str.utf8][0, 100];            # a buffer: send waits only when full
channel.send[lend 'c', give 'line'];             # a value is given, never lent
let 'v' := channel.take[lend 'c'] otherwise …;   # waits; fails `closed` when no sender is left
```

**Capacity by the bound rule** (owner: b): no bound is a hand-off (Go's
default — a meeting point, not a box); `[0, N]` is a bounded queue.
Unbounded is refused: the classic way to hide a slow consumer until the
machine dies. `take` fails `closed` exactly as `std::input.line[]`
does at end of input. A `send` nobody will ever take is a check site
(Precog sees every taker). Two tasks each waiting on the other is a
check site too — a deadlock Precog can see.

## `whichever` — whichever comes first (owner: a; the word, owner)

```
whichever {
    'msg' from lend 'inbox'     { handle[give 'msg']; }
    'why' from lend 'shutdown'  { return; }
}
```

Waits until any listed channel has a value, takes from that one, runs
its arm; each arm names its value. All arms `closed` fails `closed`.
Proposed alternatives: a std operation returning a generic enum (ugly);
merge channels into one of an enum by hand (what people would do
without it). The word: first `pick`; owner asked why not `pick_first`
(no other way to pick; no underscored keywords) and for other words —
`whichever`, `await`, `select`, `wait`, `listen`, `first` — and chose
**`whichever`**: the English for what it does.

## Time — std, with duration literals (owner)

Time is the outside world, so it is std's, like input:

```
std::time.wait[5 seconds];
let 'deadline' := std::time.after[500 millis];     # a channel that sends once
let 'tick' := std::time.every[1 second];           # a channel that sends every second
whichever {
    'msg' from lend 'inbox'     { … }
    'now' from lend 'deadline'  { std::printu.stdout["timed out", \n]; }
}
```

A number followed by a unit word is a **duration literal** — a
`std::duration` value that can be stored, passed and compared. The
unit sits on the number, as a bound's does (`[1, 40 graphemes]`; owner
asked why not, over `[1, seconds]`). Units: `nanos`, `micros`,
`millis`, `seconds`, `minutes`, `hours` — down to what a clock can
measure, stopping before the calendar (a day is not always 24 hours).
Singular and plural both accepted (`1 second`, `2 seconds`) — English
grammar, not an alias. One representation underneath (nanoseconds in
an integer wide enough for centuries); `5 seconds == 5_000 millis`.

**A duration is never negative** (owner, 2026-09-22): it is a length of
time, and every use here is a wait, a timeout or a tick. `5 seconds -
6 seconds` is refused — a check site like any narrowing past a bound,
an error when Precog sees it is certain. Proposed alternatives: a
signed duration (`-1 second`, which the integer underneath gives for
free, and which would be the natural way to say "how far past the
deadline" — but that is a difference of two *instants*, a type not
designed); no subtraction between durations at all. The rule was in
the oracle from the start and had never been asked; the compiler
session found it missing from the record.

## A channel's layout (owner, 2026-09-23)

A channel's capacity is always known when it is made — no bound is a
hand-off with no slots at all, `[0, N]` is exactly N, and unbounded is
refused — so **a channel never grows**, and its **slots sit inline**:
one block holding the bookkeeping and then N slots of `T`. One
allocation, not two. (Measured on an M5, C, with barriers: reading a
slot inline versus through a pointer is 2.44 ns against 2.47 ns —
nothing. The reason to choose inline is the allocation count, not the
access.)

**Precog's three parts**, each the usual "pay for what you use":

- **The stack, not the heap.** A structured task cannot outlive the
  block that started it, and a detached one cannot *lend* anything
  (it takes only `give` / `copy`) — so a channel lent only to ordinary
  `concur` tasks provably dies with its block and lives in the stack
  frame. Measured: creating and destroying a channel costs 13–19 ns on
  the heap against 0.9 ns on the stack, which is worth most where
  channels are made in a loop (one per connection).
- **Fewer slots than the type allows.** `[0, 100]` is the writer's
  ceiling, not a measurement; where Precog proves the channel never
  holds more than 3 at once, it allocates 3.
- **No channel at all.** One sender, one taker, one value, all inside
  one block: the hand-off becomes a plain variable the task writes
  into. Worth more than the 2.7× a microbenchmark shows, because it
  also removes the atomics and the scheduler round trip that a real
  send and take involve.

## Runtime

Tasks run on the thread pool `parallelism.md` already has — as
lighter tasks multiplexed over it, so a thousand waiting connections
do not cost a thousand threads. An implementation choice, not syntax;
the oracle runs tasks cooperatively in source order — a task runs when
something waits on it or when its block ends — which is one valid
schedule (its README lists what it simplifies).

## Open

- Cancellation beyond "a failing block cancels its tasks": can a task
  be cancelled by hand, and what does it see.
- Whether `whichever` may also `send` (Go's `select` can).
