# Diagnostics — what the compiler says

> Revised 2026-10-07: code here is in the new syntax; an old spelling in the prose is history. The old → new table is `revision-2026-10-07.md`.

Decided 2026-09-22 (owner): "It should say what started the fail chain,
it should be helpful like Rust's, there should be a human-readable
location and an IDE-readable location." The oracle's one-line messages
are a test tool's; CCA's are these.

**Columns** (owner, 2026-10-06): the text message counts graphemes, what
a reader counts; the JSON counts in the unit the editor asks for, UTF-16
code units by default, as the Language Server Protocol does. This is
Brainstem's rule (the owner then: "we can't force the other codextras to
use this anyways"; there are no others since 2026-10-07).

## The shape — plain text is the message (owner, 2026-09-22)

A drawing with carets under the source breaks in a proportional font,
a log, a chat paste, a narrow terminal, and under Thai (`เกี่ยว` is
six code points and four graphemes; a `^^^^^` lands wrong in half the
terminals). So the **plain-text form is the message**; a source excerpt
with carets is optional decoration a terminal may add when it knows
the font is monospace and the line is ASCII-safe, computed from the
same spans the JSON carries, and never carries anything the text does
not. One fact per line, a label per line, blank lines between parts,
indentation by spaces only:

```
check: 'k' can be past the end of 'xs'
where: shop.nrv, line 31, column 20 — in `pick`, called from MAIN at line 48 of shop.nrv
code:  'xs'['k']
when:  'k' is 101 and 'xs' has 100 elements — there is no element to take

why:   'k' comes from 'n'
       'n' is read from the user at line 4, with no limit

try:   synthia run shop.nrv --input "101"

fix:   limit the number where it is read — line 4:
       let 'n' := to.int32[1, 99]['text'] otherwise 0;

   or  check 'k' before using it — line 31:
       if 'k' </== list.length[lend 'xs'] { let 'x' := 'xs'['k']; … }
```

```
error: 'xs' is an `array`, which cannot grow or shrink
where: shop.nrv, line 12, column 5 — in `restock`
code:  list.append[lend.mut 'xs', 4]

why:   'xs' was declared an array at line 10

fix:   declare it a list — line 10:
       let 'xs': list[int32] = [1, 2, 3];
```

```
note:  the `for` at line 9 runs on the GPU
why:   3,000,000 iterations; the body is proven independent

note:  the `for` at line 22 stays sequential
why:   its body calls `write` (extern) at line 24
```

The labels, always in this order: the kind (`error` / `check` /
`note`) with the sentence; `where` (file, line, column, and the
human-readable place: which function, called from where); `code` (the
piece, quoted); `when` (the counterexample, as a sentence); then
`why` (the root of the chain, one fact per line); `try` (a command
that reproduces it — `synthia run … --input "…"`, a proposed Synthia
flag that feeds the counterexample as the program's input); `fix` (the
edited line, whole, with "line N:" before it; alternatives under
`or`). An error has no `when` / `try`. A reader learns the shape once.

Earlier drafts, for the record: one-liners (owner: "less useful than
C's"); rustc's caret drawing with `= note:` furniture (owner could not
read it); the same as dense prose (owner: "a blob"). This shape is
what came out.


**A borrow conflict points at the change** (owner, 2026-09-29: a): when
what a borrow borrows is changed, given away or lent mutably while the
borrow is still used later, `where` is the change, the line to move or
fix, and the message names the later use:

```
error: 's' cannot be changed here: 'r' still borrows it and is used on line 25
where: shop.nrv, line 24, column 5 — in MAIN
code:  list.append[lend.mut 's', 4]
```

Proposed alternative: the later use.

## The rules

1. **The root, first.** When one thing causes a chain — an unbounded
   read at line 4 makes a narrowing at line 20 unproven which makes an
   index at line 31 unproven — the message is about the *root*, names
   it (`why:`), and lists what it drags along, once. Never twenty
   messages from one cause; never the last link without the first.
   Cortex knows the chain because it built it: the unproven domain at
   the root is the same domain object at every site downstream.
2. **Helpful like Rust's.** Every message has: the sentence (what was
   found, why it matters, what to write instead); the code quoted; a
   second place named when the problem is between two (declared here /
   used there); `fix:` with the **fix as edited code**, never a
   description of one; for a check, the
   counterexample as real input values and the exact command that
   reproduces it.
3. **Two locations, always.** A human-readable one — *where in the
   program's own terms*: "in `pick`, the loop starting at line 9 of
   shop.nrv, called from MAIN at line 48 of shop.nrv" (every place it
   names gives its file, owner 2026-10-08: "yea") — and an IDE-readable
   one, `file:line:col`,
   plus a machine format (`--message-format json`, as rustc: severity,
   spans with byte offsets, the help edits as text replacements) so an
   editor can underline, jump, and apply the fix.
4. **Three kinds, no fourth.** `error` — will not build; `check` — the
   tiers: an unproven site, with its counterexample (an error in
   `check`-tier and final builds, a runtime check in a working build);
   `note` — the report: a proven range, a loop offloaded or not and
   why, a `!check` listed. No "warning" that lives
   forever ignored: a thing worth saying is one of the three.
5. **Plain words.** The writer's own names, quoted; the exact spelling
   to type. No jargon — not "SMT", "unsat", "lvalue", "monomorphised",
   "borrow checker". A reason in a short clause, no dates (the oracle's
   "(decided 2026-09-20)" stays in the oracle).
6. **Tone.** Errors and checks are dry: a person mid-bug wants the fix.
   Notes may carry the language's personality (the long
   `print.useful` name).
7. **Colour** in a terminal, plain text in a file, JSON when asked.
8. **Say what would happen, and no borrowed furniture.** "'k' can be
   past the end of 'xs'", not "out of bounds"; "there is no element to
   take", not "index error". Labels are plain words at the margin —
   never rustc's `= note:` (`=` means assignment here). Every mark on
   the screen means something in Cyborg. Fixes are shown as the edited
   source line, whole, not as fragments. The test of a message is
   whether the owner can read it cold.
9. **Nothing depends on alignment.** The message must say everything
   as text that survives any font, width, or paste. Carets are optional
   decoration; JSON is the machine form.

## The report

Beside the three lists (`!check` sites, `check` sites passed, default
sites still checked), a line per `for` Cortex considered (owner,
2026-09-22): `offloaded to GPU`, `parallel on N CPU cores` ("CPU" added
2026-10-06, since a core is also a compiler mod), `sequential —
body calls extern at line 14`, `sequential — 40 iterations, below the
launch threshold`, `sequential — for seq`. A writer can always see why a
loop runs where it runs.

## Open

- The exact JSON schema (follow rustc's / LSP's shapes unless a reason
  appears).
- Whether `check` messages in a working build are printed at build
  time, run time, or both (today: build time; the trap at run time
  names the site).
