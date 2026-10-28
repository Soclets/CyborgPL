# Concurrency — `concur`, channels, `whichever`

> Revised 2026-10-07: code here is in the new syntax; an old spelling in the prose is history. The old → new table is `revision-2026-10-07.md`.

Decided 2026-09-22, one question at a time. Not parallelism
(`parallelism.md`: the same work on many pieces, the prover spreads a `for`
over cores or a GPU, the programmer writes nothing). Concurrency is
*different* things that wait on the outside world — ten connections, a
download beside a keyboard — where the waiting is the point.

```
let $results: %channel[str.utf8] := %channel.empty;
let $a := "a.com";
%spawn.task with [lend $results, give $a]:
    %channel.send(lend $results, value: fetch(lend $a));
;%spawn.task   # a call's result is fresh: no verb
%spawn.task with [lend $results]:
    %channel.send(lend $results, value: fetch("b.com"));
;%spawn.task
for $i [from: 1, to: 2, by: 1]:
    let $page := %channel.take(lend $results) ?? "";    # waits; whichever finished first
    std::print(std::str.bytes(lend $page), newln);
;for
```

## The model — tasks and channels (owner: b)

A word starts a task; tasks talk through channels; a value sent is
`give`n, so no two tasks ever hold one thing — no locks, because nothing
is shared, and the prover checks that nothing is. Ownership does the work
the language already paid for. Proposed alternatives: the prover overlaps
waiting calls automatically (would rarely fire — real concurrent
programs are about shared state); `async`/`await` (colours every
caller).

## `concur` (owner, over `task`)

```
%spawn.task:
    …
;%spawn.task                       # a block, run alongside
%spawn.task fetch(lend $url);          # a call, run alongside
```

A bare word in front of a statement, the family of `check` / `no_check`.
The first six letters of *concurrent*; the owner weighed the English
reading ("agree") against `task` and chose `concur`.

## Captures — explicit, all verbs (owner: a)

A task can outlive the code around it, so it names what it takes, as an
anonymous function does (`functions-as-values.md`): `with [give 'url',
copy 'n', lend 'results']`. `give` moves it in; `copy` duplicates;
**`lend` is a check site** — the prover proves the task ends before the
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
%spawn.task with [give $url]:
    let $p := fetch($url)?; …
;%spawn.task
    otherwise:
        case no_network:
            std::print("offline", newln);
        ;case no_network
    ;otherwise;
```

A task that can fail says so where it is written, like every failing
call; the handler runs **when the enclosing block ends**, the point at
which the task's result is known. Proposed alternatives: `otherwise` on
the block; tasks cannot fail (plumb it through a channel by hand).

**A detached task has no such block**, so its handler runs **inside the
task, the moment it fails** (owner, 2026-09-23):

```
proc setup [give $doc: Document]:
    %spawn.task.detached with [give $doc]:
        save(lend $doc)?;
    ;%spawn.task.detached
        otherwise:
            case disk_full:
                …
            ;case disk_full
        ;otherwise;        # runs here, in the task, at once
;proc setup
```

It can print, log, or retry; it cannot `return` or fail outward, since
there is nobody left to return to. The body's own work is unwound
first, but **the captures are still alive while the handler runs**, so
a retry has what it was given (kayo, 2026-09-23). Proposed alternatives: a detached
task may not fail at all (the checker refuses `fail` inside one); its
failure ends the program with the reason named. The autosave that could
not write is why detached tasks exist, and it should be able to say so
while the program carries on.

## Channels

```
let $c: %channel[str.utf8] := %channel.empty;                    # hand-off: send waits for a taker
let $q: %channel[str.utf8][max: 100] := %channel.empty;            # a buffer: send waits only when full
%channel.send(lend $c, value: give $line);             # a value is given, never lent
let $v := %channel.take(lend $c) ?? …;   # waits; fails `closed` when no sender is left
```

**Capacity by the bound rule** (owner: b): no bound is a hand-off (Go's
default — a meeting point, not a box); `[0, N]` is a bounded queue.
Unbounded is refused: the classic way to hide a slow consumer until the
machine dies. `take` fails `closed` exactly as `std::input.line[]`
does at end of input. A `send` nobody will ever take is a check site
(the prover sees every taker). Two tasks each waiting on the other is a
check site too — a deadlock the prover can see.

## `whichever` — whichever comes first (owner: a; the word, owner)

```
%whichever:
    case $msg from lend $inbox:
        handle(give $msg);
    ;case $msg
    case $why from lend $shutdown:
        ret;
    ;case $why
;%whichever
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
std::time.wait(for: 5 secs);
let $deadline := std::time.after(500 millis);     # a channel that sends once
let $tick := std::time.every(1 sec);           # a channel that sends every second
%whichever:
    case $msg from lend $inbox:
        …
    ;case $msg
    case $now from lend $deadline:
        std::print("timed out", newln);
    ;case $now
;%whichever
```

A number followed by a unit word is a **duration literal** — a
`std::duration` value that can be stored, passed and compared. The
unit sits on the number, as a bound's does (`[1, 40 graphemes]`; owner
asked why not, over `[1, seconds]`). Units: `nanos`, `micros`,
`millis`, `secs`, `mins`, `hours` (`seconds` and `minutes` until 2026-10-24, owner: "we might wanna change it to
`secs` & `mins`", then "yes"; `wait` says `for:`, its duration a setting; `after(5 secs)` and `every(1 sec)` stay bare, their names the word,
owner: "keep those bare") — down to what a clock can
measure, stopping before the calendar (a day is not always 24 hours).
Singular and plural both accepted (`1 sec`, `2 secs`) — English
grammar, not an alias. One representation underneath (nanoseconds in
an integer wide enough for centuries); `5 secs equal_to 5_000 millis`.

**A duration is never negative** (owner, 2026-09-22): it is a length of
time, and every use here is a wait, a timeout or a tick. `5 secs -
6 secs` is refused — a check site like any narrowing past a bound,
an error when the prover sees it is certain. Proposed alternatives: a
signed duration (`-1 second`, which the integer underneath gives for
free, and which would be the natural way to say "how far past the
deadline" — but that is a difference of two *instants*, a type not
designed); no subtraction between durations at all. The rule was in
the oracle from the start and had never been asked; the compiler
session found it missing from the record.

## When a channel is closed (owner, 2026-09-23: c)

`take` waits for a value; if nobody will ever send again, waiting for
ever is a hang, so it fails `closed` instead. How that is known:

**A channel counts the tasks that can reach it** — the one that made
it, plus each task it was lent or given to — and a task drops out when
it finishes. A `take` on an empty channel fails `closed` when every
task still holding it is itself waiting to take from that same
channel: nothing can arrive, so the answer is exact rather than a
guess. Two counters per channel, updated as tasks start and end. The
case it does not answer is a holder stuck waiting on something else,
which keeps the channel open and the take waiting — that is a
deadlock, already a check site above.

**And the prover deletes the counting wherever it proves the last sender
is done** — structured concurrency tells it when every task that could
send has ended, so the close is emitted at that point and the counters
are not built. Proposed alternatives: the counting alone; the prover alone
(it will not prove every case, so something is still needed for the
rest). Owner: both, the counting built first.

## A channel's layout (owner, 2026-09-23)

A channel's capacity is always known when it is made — no bound is a
hand-off with no slots at all, `[0, N]` is exactly N, and unbounded is
refused — so **a channel never grows**, and its **slots sit inline**:
one block holding the bookkeeping and then N slots of `T`. One
allocation, not two. (Measured on an M5, C, with barriers: reading a
slot inline versus through a pointer is 2.44 ns against 2.47 ns —
nothing. The reason to choose inline is the allocation count, not the
access.)

**the prover's three parts**, each the usual "pay for what you use":

- **The stack, not the heap.** A structured task cannot outlive the
  block that started it, and a detached one cannot *lend* anything
  (it takes only `give` / `copy`) — so a channel lent only to ordinary
  `concur` tasks provably dies with its block and lives in the stack
  frame. Measured: creating and destroying a channel costs 13–19 ns on
  the heap against 0.9 ns on the stack, which is worth most where
  channels are made in a loop (one per connection).
- **Fewer slots than the type allows.** `[0, 100]` is the writer's
  ceiling, not a measurement; where the prover proves the channel never
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
