# A licence that defines every word

An idea of the owner's, 2026-10-02, noted for later. The owner's words:

> Another fun thing, just note down for now. Making a license that
> describes every word used there, software

A software licence in which every word it uses is defined inside it.

Design claude's note: real licences define only their key terms (Apache
2.0's section 1 defines "License", "Licensor", "Work", "Contribution" and
a few more). Defining *every* word runs into the words used in the
definitions themselves — the fun part — so it would have to end at some
base set of words, or define its words in terms of each other in a
closed loop.

The owner, next: "Since, that might take long. It should run on a quite
machine effieciently", then "I just meant that the software should be
optimized". Design claude's reading: defining every word is a long job,
so the program that builds the licence should be optimised to do it
efficiently.

The owner, on how: "Definition is not by AI, it's by the software. So, it
would have to get definitions of every word used, including the one used
in the definitions". So the program looks every word up, then every word
in those definitions, and so on until nothing new comes up, and the
licence carries all of it.

**The dictionary is Wiktionary, and it runs on the owner's gaming PC**
(owner, 2026-10-02: "Wiktionary it is, on my gaming PC (Ryzen 7 9700X, RX
9070 XT, 32GB 5600MT/s RAM, 2TB Gen4 NVMe storage), not MacBook"). Design
claude had offered WordNet (155,287 words, 10.5 MB, permissive, but no
grammar words such as "the", "of", "if") against Wiktionary (9,201,874
English-edition pages on 2026-10-02, a 1.65 GB compressed dump, every
word). What comes with it, as design claude explained beforehand:
Wiktionary is CC BY-SA, so a licence carrying its definitions is shared
under CC BY-SA, with credit; its dump is raw wiki markup, so the program
needs a real parsing step; and a fixed dump with its checksum, not the
live site, keeps runs reproducible and vandalism out.

**The program is in Rust** (owner, 2026-10-02: "Well, a license is
supposed to be correct, Rust"; design claude had suggested it).

**It runs on the Mac after all** (owner, 2026-10-02): "Since that's one
of the biggest projects too. I'll say on my Mac, instead of the PC" —
"that" being CCOESS, which the PC route would have needed — then "No, I
meant the license software". Design claude's note: on the Mac it shares
24 GB with the other sessions (RRx2-TV may hold up to 18 GB, `CLAUDE.md`),
so it should read the dump as a stream rather than unpack it whole, and
run when the Mac is quiet.

**OLLGS — Optimized Long License Generator Software** (owner,
2026-10-02). Built by another session, **troll license claude** (owner's
name), in a public repo under the org (owner: "for fun, this shit is
gonna be under the org. Public"): `Cyborg-Assembly-Manufacturing-Site/OLLGS`,
created empty the same day. README, warning and the code's licence: "no,
not picked yet". Its resources, asked about cores and memory with
RRx2-TV stopped: "unbounded". Its brief and prompt: not yet ("Don't make
the brief & prompt yet").
