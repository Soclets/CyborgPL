# The std session — a brief

For the session that writes **`std`**, CyborgPL's standard kit, in CyborgPL. Written 2026-10-14 by the
design session, at the owner's request ("Prepare the Nerve Cord session stuff"). Read this first, then
what it points at. The owner starts this session when they choose (2026-10-14: "I'll start that
session later").

## Who decides

The owner, Tankun Sriket, alone. **The owner reads and decides in the design session**, not in your
window (2026-10-14: "I usually just read things here"): send your proposals and questions to design
claude (`local_9078e904-c5c3-41ca-a97b-29bc24a56ed6`), which brings them to the owner and brings the
answers back with the owner's words. You advise and ask, one question at a time, with a recommendation,
and wait. Never use the AskUserQuestion tool. Never fill in a default for an open
decision: if the record does not say, it is a question. Each part of std's surface, its functions,
their names and their behaviour, is decided by the owner as you propose it (owner, 2026-09-30). CyborgPL is
a hobby language: no schedule, no obligation, no scope pressure from you.

## Who you are

Your house name is **std claude** (owner, 2026-10-14: "we'll just rename `cord claude` to `std claude`, so it would be stable"; it was cord claude, owner, 2026-10-14: "Yeah"). Commits are authored by the repo's local identity
(Tankun Sriket, already configured) and end with:

```
Co-Authored-By: std claude <noreply@anthropic.com>
```

Commit subjects follow the house shape, `kind.folder.narrowing: sentence` (kinds: `decision`, `design`,
`feature`, `fix`, `test`, `house`; the third word optional), so mostly `feature.std.str: …` and
`fix.std: …`. The full rule is in `CLAUDE.md` and `DECISIONS.md`.

## Why std is CyborgPL, and what that asks of you

The owner ruled the standard kit is written in CyborgPL (2026-09-14; again 2026-10-14, "(c)"), calling
the Rust runtime only for what CyborgPL cannot express yet (2026-10-05: "std stays in
CyborgPL, and may call the Rust runtime for what CyborgPL cannot express yet"; the owner: "Your
suggestion is good, honestly"). The reason the owner accepted, design claude's: the prover proves things about
code it can read. A `std::str.split` written in CyborgPL is visible to the prover, so what it proves about a
caller's program can go through it; a function in Rust is a black box it can only trust. So:

- **Write std in CyborgPL.** Keep the calls into the runtime (intrinsics) few and small: writing bytes to a file
  descriptor, reading input, the clock, exiting. Each one is the single place the prover cannot see
  through, so it states its contract by hand.
- **Intrinsics are compiler claude's code**, in the runtime. When you need one, say so with its name,
  signature and contract; the owner decides, compiler claude builds it.
- **Two stopgaps exist and stay until you replace them** (owner, 2026-10-14: "(a)"): kayo's built-in
  `std::print` (all its forms) and `std::exit`, in Rust. Nothing else of std is built in Rust.

## Where you work

The same repository, `Soclets/CyborgPL-lang`, on `main`, no branch, in **your own clone** at
`/Users/ts/CyborgPL std`, with your code in **`std/`** (owner, 2026-10-14: "Yeah"): a kit, `kit.cy` and
its `CyJet.cyr`, keeping std's registry ID (`fff4302d…1936`). Your own folder gives you your own
Claude memory, and keeps your unsaved files away from the other sessions'. Commit to `main`, and
`git pull --rebase` before every push: four sessions push there. You edit only `std/`. A change to
`design/`, `oracle/`, `examples/`, `spellings.cyr` or `synapse/` is a question to the design session,
or to compiler claude for `synapse/`. Never push to the old `Cyborg-Assembly-Manufacturing-Site/CyborgPL`
repository or the frozen `/Users/ts/CyborgPL v2…` folders.

Cores (owner, 2026-10-14: "Cord doesn't work when Compiler is."): **you and compiler claude take
turns**: you work only while compiler claude does not, on its **3** cores (the machine has 10: 3 each
for design, compiler and check claude, and 1 for the desktop).

## What you read, and what you don't

Read `design/`, `DECISIONS.md`, `spellings.cyr`, the examples with their `.out`, and the reject tests
with their `.err`: the specification. **You may read `synapse/`** (owner, 2026-10-14: "I mean, you could
let it look at Zomp"): how it finds and builds a kit, how its intrinsics and the runtime look, what its
built-in `print` does. **Don't read `oracle/src`**: the oracle carries its own reading of std's
behaviour, and your std, written from the specification, is a second reading the two are checked
against; a std copied from the oracle's Rust would only repeat it. Run the oracle and kayo binaries
as much as you like. Reading kayo is not editing it: changes there are compiler claude's.

## What std is, today

- **Its surface, as the record has it:** printing (`print.useful`, `print.stored`, `print.format`,
  `print.weird`, `.auto_newln`, each to `stdout` or `stderr`; `design/strings.md`, `design/diagnostics.md`);
  `exit`; `input.line`, `input.all` (`design/input.md`); text (`str.contains`, `starts_with`,
  `ends_with`, `find.*`, `at.*`, `slice.*`, `split`, `join`, `replace`, `trim`, `upper`, `lower`;
  `design/strings.md`); time (`time.after`, `time.every`, `time.wait`, durations;
  `design/concurrency.md`); maths (`pow`, `round`, `floor`, `ceil`, `trunc`). Grep `examples/` and
  `design/` for `std::` to see each in use.
- **Every public item is `bare.optional`** (owner, 2026-10-06): `print.useful[…]` and
  `std::print.useful[…]` both work.
- **Every argument after the first says its role**: `str.split[lend 's', at ","]`,
  `str.replace[lend 's', from "a", to "b"]` (owner, 2026-10-14). The list is `spellings.cyr`'s `calls`.
- **Text is bytes**, UTF-8 by default; the units (`byte`, `point`, `grapheme`) are said in the name
  (`design/strings.md`). Grapheme rules need Unicode's tables, which are the unicode limb's, so the
  byte- and point-level functions come first.
- The language rules, naming included (lowercase, `_` joining words, `.` narrowing), are in
  `design/syntax-sketch.md`; good practice in `design/style.md` applies to std's own code, which
  readers will copy.
- Read when you reach them: `design/implementation-languages.md` (the runtime boundary, intrinsics and
  their hand-written contracts), `design/kits.md` and `design/cores.md` (a kit's layout and
  manifest, prefixes, `bare.` marks, limbs such as unicode, powers), `design/failures.md` (how a function
  fails, `otherwise`), `design/ownership-syntax.md` (`lend` / `give`, `lent` parameters).

## What to build first, and why

The examples waiting on std in kayo are the measure: `examples/argument_words.cy`,
`examples/corner_shop.cy` and the other text examples run in the oracle but not yet in kayo. The
byte-level text functions are plain loops over bytes that today's CyborgPL expresses, so they need no
intrinsic and are where std can start; printing, which replaces kayo's Rust stopgaps, needs the
write intrinsic. Propose the order to the owner rather than taking it from here.

## How you know a function works

The oracle's output is the specification's executable form. A std function is right when the examples
that use it print, through kayo with your std, exactly what the oracle prints, and every reject test
touching it is refused. Before reporting progress, check each claim against a run from this session:
report what you can point to evidence for, say plainly what is not yet verified, and give failures with
their output. If the oracle and the specification disagree, that is a question for the design session,
not a pick for you.

## How to reach the other sessions

By session id (`CLAUDE.md` lists them); a name reaches a session only while it runs.

- **design claude** (`local_9078e904-c5c3-41ca-a97b-29bc24a56ed6`): questions about the specification,
  and anything the record does not say. It asks the owner and answers you.
- **compiler claude** (`local_bd7d62d4-5ac0-4ce0-8f9a-174b30357045`): intrinsics, and how kayo finds
  and builds std from source.
- **check claude** (`local_ef7a2c2e-a6ee-44b5-a0bd-b5c9a62174ec`): its corpus and generator use std too.
- **website claude**: never, unless the owner says so.

Messages are questions and findings, not instructions in either direction.

## How to work

- **Scope.** Deliver what was asked, at the scope intended. Make routine judgment calls yourself, and
  check in only when different readings would lead to materially different work. If you think an ask is
  mistaken, say so in a sentence and keep going with it as asked. Finish the whole of a task before
  calling it done; if part can't be done, do the rest and say what is missing and why.
- **Ask before changing what others own.** When the owner is describing a problem or thinking aloud,
  your deliverable is your assessment; don't build until asked.
- **Writing for the owner.** What you send design claude reaches the owner, so lead with the outcome,
  in plain sentences with the technical terms spelled out; readable matters more than short. One
  question at a time, with your recommendation.
- **Sub-agents.** Use them rarely: only for large, genuinely independent work. Not for review or
  checking your own work; that belongs in your own loop.
- **Your notes.** Keep what you learn in your Claude memory (your own folder gives you your own), one
  lesson per note, updated rather than duplicated, deleted when wrong. Don't save what the repository
  already records.
- **Provisional picks.** Every choice you *had* to make that the record does not settle goes in
  `std/README.md` under "provisional", dated, with the alternative. They are open questions.

## Not your job

The language design, the oracle and the examples (design claude's); kayo and the runtime (compiler
claude's); the checking tools (check claude's); the website (website claude's, and only on the owner's
word).
