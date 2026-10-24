# Diagnostics — what the compiler says

> Revised 2026-10-24: the message is its name, the code and the fix (owner, 2026-10-22); code here is in the 2026-10-22 syntax (calls in `( )`, `$` names).

Decided 2026-09-22 (owner): "It should say what started the fail chain,
it should be helpful like Rust's, there should be a human-readable
location and an IDE-readable location." The oracle's one-line messages
are a test tool's; kayo's are these.

**Columns** (owner, 2026-10-06): the text message counts graphemes, what
a reader counts; the JSON counts in the unit the editor asks for, UTF-16
code units by default, as the Language Server Protocol does. This is
the base's rule (the owner then: "we can't force the other codextras to
use this anyways"; there are no others since 2026-10-07).

## The message — its name, the code and the fix (owner, 2026-10-22; not built)

```
error[unclosed_name] at shop.cy:3:9
where: in pick(), called from take_order() at line 240
code:  let 'a := $b + 1;
fix:   let 'a' := $b + 1;
```

```
check[index_out_of_range] at shop.cy:31:20
where: in pick(), called from MAIN at line 48
code:  $xs[pos: $k]
when:  $k is 101, and $xs holds 100
fix:   line 4:
       let $n := to.i32[min: 1, max: 99]($text) otherwise 0;
```

- **An error is named, and the name is its title**: `error[unclosed_name]`, `error[borrow_outlives_owner]`,
  `check[index_out_of_range]`. One name per kind of error, written once and never composed for a message, so it cannot
  mislead (the owner, shown a message composed wrong for a quote left open mid-line: "Or only add the code too, so
  it's less surface to be wrong"; "yup" to a fixed title per error). Asked (a) codes like `E0107`, (b) codes by area,
  (c) a name that is the title (recommended): "Or just some openssl rand magic?", then the names went ahead.
- **A limb's errors carry its prefix**: `error[cnc::opaque_struct_held]`, `error[metal::two_kernels]`; the base's have
  none ("That `::` is a great idea"). Names never clash, as kits' do not.
- **The message is the code and the fix**: `code:` the piece quoted, `fix:` the fix as edited code, never a
  description of one ("Where's the fix line?", then "Or do we just show code, and fix only? While, learning could be in
  docs"). **A check adds `when:`**, the value that breaks it. There is no sentence, no `why:` and no `try:`.
  **Every edit kayo can write is shown, under `or`, nearest the error first** (owner, 2026-10-24: "show them all");
  where the fix lies outside the program's own code, the line is `fix: none suggested`, so every message has its `fix:`.
- **The explanation is in the docs**, found by the error's name.
- **The oracle and kayo use the same names, from one list** (owner, 2026-10-24: "yea"; not built): the oracle's
  one line starts with the name, so a test checks both engines refuse a program for the same reason.
- **Two locations** ("There should be two location thingies, one for humans, and one for programs"): `file:line:col`
  on the first line for programs, and `where:` for people, **a function written with `( )`**, `pick()` (the owner:
  "`pick()`", over backticks, a backtick being CyborgPL's char literal: "` means char in CyPL").
- Design claude's readings (2026-10-24, compiler claude asking): `where:` outside any function reads `where: in struct
  Item`, `where: at the top of shop.cy`, `where: in CyJet.toml, my kits:`; `called from f() at line N` only on a check
  that depends on its caller, otherwise `where: in pick()`, a loop as `where: in pick(), the loop starting at line 9`;
  a concise file's place as `where: in pick(), decoded line 12 of shop.cy (concise)`.
- Design claude's readings, not asked: the layout (the name, `at`, the location, then `where:`, `code:`, `when:`,
  `fix:`); a fix on another line as `fix: line 4:` with the line under it, alternatives under `or`; a renamed error
  keeps its old name as an alias, as `spellings.cyr` keeps old spellings; the `when:` wording.

What stays from the 2026-09-22 shape (owner): **the plain text is the message**. A drawing with carets under the
source breaks in a proportional font, a log, a chat paste, a narrow terminal, and under Thai (`เกี่ยว` is six code
points and four graphemes); so carets are optional decoration a terminal may add when it knows the font is monospace
and the line is ASCII-safe, computed from the same spans the JSON carries, never carrying anything the text does not.
One fact per line, a label per line, indentation by spaces only.

What it replaced, for the record: the 2026-09-22 shape opened with a sentence (`check: 'k' can be past the end of
'xs'`) and had `why:` (the root of the chain), `try:` (a command reproducing it) and `fix:`; earlier still, one-liners
(owner: "less useful than C's"), rustc's caret drawing with `= note:` furniture (owner could not read it), and dense
prose (owner: "a blob").

**A borrow conflict points at the change** (owner, 2026-09-29: a): when what a borrow borrows is changed, given away
or lent mutably while the borrow is still used later, the location is the change, the line to move or fix:

```
error[changed_while_borrowed] at shop.cy:24:5
where: in MAIN
code:  std::list.append(lend.mut $s, value: 4);
fix:   line 26:
       std::list.append(lend.mut $s, value: 4);
```

(the fix moving the change below the borrow's last use, line 25; design claude's sketch in the new shape.)

**What the engines print today** (2026-10-24): the oracle one line, `file: line L:C: message`, a test tool's; kayo
`error:` with a sentence, `where:` and `code:`, no name and no `fix:` yet.

## The rules

1. **The root, first.** When one thing causes a chain — an unbounded
   read at line 4 makes a narrowing at line 20 unproven which makes an
   index at line 31 unproven — one message is given, at the site, and its
   `fix:` is at the root (`fix: line 4:`). Never twenty messages from one
   cause. The prover knows the chain because it built it: the unproven
   domain at the root is the same domain object at every site downstream.
   (Until 2026-10-22 the root had its own `why:` line.)
2. **The code and the fix** (owner, 2026-10-22). Every message has its
   name, the code quoted, and `fix:` with the **fix as edited code**,
   never a description of one; a check, `when:`, the values that break
   it. What the error means, and why, is in the docs under its name.
   (Until 2026-10-22: a sentence, and a command reproducing a check.)
3. **Two locations, always.** A human-readable one — *where in the
   program's own terms*: "in pick(), the loop starting at line 9 of
   shop.cy, called from MAIN at line 48 of shop.cy" (every place it
   names gives its file, owner 2026-10-08: "yea") — and an IDE-readable
   one, `file:line:col`,
   plus a machine format (`--message-format json`, as rustc: severity,
   spans with byte offsets, the help edits as text replacements) so an
   editor can underline, jump, and apply the fix.
4. **Three kinds, no fourth.** `error` — will not build; `check` — the
   tiers: an unproven site, with its counterexample (an error in
   `check`-tier and final builds, a runtime check in a working build);
   `note` — the report: a proven range, a loop offloaded or not and
   why, a `no_check` listed. No "warning" that lives
   forever ignored: a thing worth saying is one of the three.
   **Every note is in the build report only** (owner, 2026-10-22: "But all
   should note in report only?"); the terminal shows errors, checks and the
   one summary line, and the report's section is **Notes**, a note reading
   `note: the \`for\` at line 9 runs on the GPU` with its reason under it.
5. **Plain words.** The writer's own names, as written (`$k`); the exact
   spelling to type, in `fix:`. No jargon — not "SMT", "unsat", "lvalue", "monomorphised",
   "borrow checker". A reason in a short clause, no dates (the oracle's
   "(decided 2026-09-20)" stays in the oracle).
6. **Tone.** Errors and checks are dry: a person mid-bug wants the fix.
   Notes may carry the language's personality (the long
   `print.useful` name).
7. **Colour** in a terminal, plain text in a file, JSON when asked.
8. **Say what would happen, and no borrowed furniture.** A name says
   what would happen, `index_out_of_range`, and `when:` the values,
   "$k is 101, and $xs holds 100". Labels are plain words at the margin —
   never rustc's `= note:` (`=` means assignment here). Every mark on
   the screen means something in Cyborg. Fixes are shown as the edited
   source line, whole, not as fragments. The test of a message is
   whether the owner can read it cold.
9. **Nothing depends on alignment.** The message must say everything
   as text that survives any font, width, or paste. Carets are optional
   decoration; JSON is the machine form.

## The report

Beside the three lists (`no_check` sites, `check` sites passed, default
sites still checked), a line per `for` the prover considered (owner,
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
