# Diagnostics — what the compiler says

Decided 2026-09-22 (owner): "It should say what started the fail chain,
it should be helpful like Rust's, there should be a human-readable
location and an IDE-readable location." The oracle's one-line messages
are a test tool's; CCA's are these.

## The shape

```
error: 'xs' is an `array`, which cannot grow or shrink
  --> shop.cpls:12:5   (in `restock`, the loop starting at line 9)
   |
10 |     let 'xs': array[int32] = [1, 2, 3];
   |               ------------ declared an array here
11 |
12 |     list.append[lend.mut 'xs', 4];
   |     ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ appended to here
   |
help: declare it a `list`:
   |
10 |     let 'xs': list[int32] = [1, 2, 3];
   |               ~~~~~~~~~~~
```

```
check: line 31 can fail — 'k' can be past the end of 'xs'
  --> shop.cpls:31:20   (in `pick`, called from MAIN at line 48)
   |
31 |         let 'x' := 'xs'[pos 'k'];
   |                    ^^^^^^^^^^^^^ if 'k' is 101 and 'xs' has 100 elements, there is no element to take
   |
why:    'k' comes from 'n', and 'n' is read from the user at line 4 with no limit — it can be anything
try it: jetpack run shop.cpls --input "101"
help:   either limit the number where it is read:
   |
 4 |     let 'n' := to.int32[1, 99][std::input.line[] otherwise ""] otherwise 0;
   |                        ~~~~~~~
        or check 'k' before using it:
   |
31 |         if 'k' <== list.length[lend 'xs'] { let 'x' := 'xs'[pos 'k']; … }
```

(The first draft said "can go out of bounds", "started at:", and put
`=` before the extra lines as rustc does; the owner could not read it.
The rule that came out of it is 8 below.)

## The rules

1. **The root, first.** When one thing causes a chain — an unbounded
   read at line 4 makes a narrowing at line 20 unproven which makes an
   index at line 31 unproven — the message is about the *root*, names
   it (`why:`), and lists what it drags along, once. Never twenty
   messages from one cause; never the last link without the first.
   Precog knows the chain because it built it: the unproven domain at
   the root is the same domain object at every site downstream.
2. **Helpful like Rust's.** Every message has: the sentence (what was
   found, why it matters, what to write instead); the source lines with
   the exact spans underlined; a second span when the problem is between
   two places (declared here / used there); `help:` with the **fix as
   edited code**, never a description of one; for a check, the
   counterexample as real input values and the exact command that
   reproduces it.
3. **Two locations, always.** A human-readable one — *where in the
   program's own terms*: "in `pick`, the loop starting at line 9, called
   from MAIN at line 48" — and an IDE-readable one, `file:line:col`,
   plus a machine format (`--message-format json`, as rustc: severity,
   spans with byte offsets, the help edits as text replacements) so an
   editor can underline, jump, and apply the fix.
4. **Three kinds, no fourth.** `error` — will not build; `check` — the
   tiers: an unproven site, with its counterexample (an error in
   `check`-tier and final builds, a runtime check in a working build);
   `note` — the report: a proven range, a loop offloaded or not and
   why, the `&` remark, a `nocheck` listed. No "warning" that lives
   forever ignored: a thing worth saying is one of the three.
5. **Plain words.** The writer's own names, quoted; the exact spelling
   to type. No jargon — not "SMT", "unsat", "lvalue", "monomorphised",
   "borrow checker". A reason in a short clause, no dates (the oracle's
   "(decided 2026-09-20)" stays in the oracle).
6. **Tone.** Errors and checks are dry: a person mid-bug wants the fix.
   Notes may carry the language's personality (the `&` remark, the long
   `printu` name).
7. **Colour** in a terminal, plain text in a file, JSON when asked.
8. **Say what would happen, and no borrowed furniture.** "'k' can be
   past the end of 'xs'", not "out of bounds"; "there is no element to
   take", not "index error". The extra lines are labelled with plain
   words at the margin — `why:`, `try it:`, `help:` — never rustc's
   `= note:` (`=` means assignment here). Every mark on the screen means
   something in Cyborg. Fixes are shown as the edited source line, not
   as fragments. The test of a message is whether the owner can read
   it cold.

## The report

Beside the three lists (`nocheck` sites, `check` sites passed, default
sites still checked), a line per `for` Precog considered (owner,
2026-09-22): `offloaded to GPU`, `parallel on N cores`, `sequential —
body calls extern at line 14`, `sequential — 40 iterations, below the
launch threshold`, `sequential — for seq`. A writer can always see why a
loop runs where it runs.

## Open

- The exact JSON schema (follow rustc's / LSP's shapes unless a reason
  appears).
- Whether `check` messages in a working build are printed at build
  time, run time, or both (today: build time; the trap at run time
  names the site).
