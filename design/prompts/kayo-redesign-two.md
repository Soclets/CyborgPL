The owner gives the go: convert kayo to the redesigned syntax, "redesign two". This is your normal work as compiler claude, under design/compiler-session.md and CLAUDE.md, which still apply in full.

<context>
On 2026-10-26 and 2026-10-27 the owner redesigned CyborgPL's surface syntax twice: blocks became `kind name:` … `end of kind name`, and then the words were shortened (`fn`/`proc`, `var`/`let`, `rec`, `union.tag`, `ret`, `int.sign.32`, `float.%deci.64`, `ls`, `eq`, `lt`, `neg`, `sig`, `set` and the rest). The oracle now reads only the new syntax, and every program in `examples/` and `oracle/tests/reject/` has been converted. kayo still reads the old syntax, so right now it fails almost every example. Until it reads the new one, the differential tests and std's conversion are both stuck behind it, which is why this comes first.

Where the rules are, in the order to read them:
- `design/revision-2026-10-27.md`: the old → new table, and the checks that came with the redesign. Start here.
- `DECISIONS.md`: every entry dated 2026-10-26 (night) and 2026-10-27, newest first. The titles start "Redesign:" or "Redesign two:", and the late 2026-10-27 ones cover `sig`/`set`, operands, C variables, `fn[…]`/`proc[…]` values, the case marks and collection members. Each entry quotes the owner and says what was built.
- `spellings.cyr`: the keywords are now the short words. The old spellings carry `since = "2026-10-27"` and are refused naming the new one, and `retired` lists the redesign-one long words.

The spec is the oracle's behaviour, read through its tests as always, never through `oracle/src`:
- `examples/*.cy` with their `.out` (and `.exit`).
- `oracle/tests/reject/*.cy`, each with an `.err` that is a substring of the oracle's message. kayo keeps its own phrases in `kayo/compiler/tests/reject_phrases.txt`.
- The reject set changed a lot. `git log --stat b82572d1^..HEAD -- oracle/tests/reject examples` shows which tests were added, removed or flipped. Some old-spelling tests stay deliberately old, to check the refusal.

The oracle binary includes a converter you can use as a tool. Running it is fine; reading its source is not.
- `oracle --redesign <file | folder>` rewrites pre-redesign code to redesign two in place.
- `oracle --redesign-show <file>` prints the result instead of writing it.
- It converted 226 of the 236 old examples cleanly. The rest needed a hand edit:
  - the case marks are gone, so a program using them has to drop them;
  - a `sig` listing types next to operations must be split into a `set` and a `sig`;
  - a template's generated part has to be regenerated;
  - a few `fn`-versus-`proc` calls can't be inferred.
- It rewrites code only, never comments, and old-spelling refusal tests must keep their old spelling. The owner's rule: "Better becareful with quoted design shit".

New checks came with the redesign, and kayo needs each one:
- A `fn` has no effects and calls only `fn`s. It may change its own locals, and one that can fail is still a `fn`. A `proc` may have effects.
- Lambdas and function types carry their kind: `fn [ … ] = …` versus `proc [ … ] = …`, `fn[…]` versus `proc[…]`. A `fn[…]` holds only `fn`s, while a `proc[…]` holds either.
- A collection's members are checked as their collection's kind, `coll.fn` or `coll.proc`.
- A `let` never changes, and a `var` that nothing changes is refused.
- Every `end of` names the kind and name its block opened with, including `end of %layout.C rec X`. An `if` chain closes once, with `end of if`.
- `!` stands exactly where the decisions say: `!MAIN`, every `!no_check`, an arm that jumps, and a jumping handler where it is used.
- `break` and `continue` outside a loop are refused at build time. You already do this.
- A `sig` and a `set` never mix.
- C variables are `%extern.C let` (C's `const`) or `%extern.C var`. Operands say `lent`, `let` or `var`, and `MAIN` says `let`.
- The case marks are removed:
  - every name is exact;
  - a name that isn't found suggests one that differs from it only in case;
  - two names in a kit that differ only in case build, with a note in the build report. Only kayo makes a build report, so the oracle has nothing here to compare against.
- The line-end `-` joins a line only to a block's opening line below it.
</context>

<task>
Make kayo read redesign two, and only redesign two:
1. Old spellings are refused, naming the new one, as the oracle and `spellings.cyr` do.
2. kayo has every check listed above.
3. kayo's own CyborgPL is converted: everything under `kayo/compiler/tests` (programs, fixes, final, refuse, bounds, testing, kits) and `kayo/bench`. Use the converter first, then fix by hand what it can't.
4. `reject_phrases.txt`, `errors.cyr`, `examples_pending.txt` and `examples_unbuilt.txt` match the new reject set and the new syntax.
5. kayo's own messages use redesign two's words.

The oracle did this as a token-level front pass ahead of its unchanged parser, which kept the change small. That's information, not a requirement: the design of kayo's front end is yours.

It's done when all of these hold:
- kayo builds and runs every program in `examples/`, printing what the oracle prints, except those listed in `examples_unbuilt.txt` or `examples_pending.txt`, each with its reason.
- It refuses every program in `oracle/tests/reject/` with its phrase.
- Its own suite passes.
- Old syntax is refused.
</task>

<how_to_work>
- Design questions go to design claude, `local_9078e904-c5c3-41ca-a97b-29bc24a56ed6`, one at a time with your recommendation. Don't decide them yourself. Carry on with whatever doesn't depend on the answer.
- Don't edit `design/`, `oracle/` or `examples/`. A wrong example or test is a question for design claude.
- Use at most 3 cores for builds, tests and runs. std claude works only while you don't, and it converts `std/` right after you finish.
- Commit in pieces as each part works, under the house rules (`kind.folder.narrowing: sentence`, your trailer, `git pull --rebase` before every push). Then a stop at any point leaves a working state.
- Work directly. Use a subagent only for a large, truly independent piece, such as converting one test folder while you change the front end, never to double-check your own work.
- A tool worth running again goes in `kayo/tools/` and gets committed. Scratch is only for what's fine to lose.
- Keep going until the done conditions hold. Don't end a turn with a summary that announces the next step instead of taking it, and don't stop to ask whether to continue. The stops that are fine: when a design question blocks everything that's left, and when you're done.
</how_to_work>

<report>
When you finish, send design claude a message saying the conversion is done, so std claude can start on `std/`. Then end with a short report that:
- leads with the outcome;
- gives the counts: examples matching the oracle, reject tests refused with their phrase, kayo's own tests passing;
- lists what's left and why;
- lists the questions you asked and their answers.
</report>
