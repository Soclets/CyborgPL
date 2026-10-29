The owner gives the go: measure Z3's Spacer as Precog's second way to find a loop's invariant. This is your normal work as compiler claude, under design/compiler-session.md and CLAUDE.md, which still apply in full.

<context>
Precog proves each check site by putting exact questions to Z3 and CVC5 (DECISIONS 2026-09-22). For a loop it needs an invariant, which today it picks from a fixed list of shapes; a loop no shape fits leaves its sites unproven (2026-09-22, "a and fallback to c"). Spacer is the Constrained Horn Clause engine inside the Z3 kayo already links, and it finds invariants by itself. On 2026-10-25 the owner asked "Should we use Z3's Spacer?" and agreed to measure it before deciding (DECISIONS, "Measure Z3's Spacer as Precog's second way to find a loop's invariant, before deciding").

Nothing is decided yet: this task produces the numbers the owner decides from. CyborgPL's stated identity is "very slow at the development, fast & safe at the final product", so a site proven is worth build time, but how much build time is the owner's call. That is why the figures need to be exact and reproducible.
</context>

<design>
The shape, from that entry:
1. Shapes first, as now. Spacer runs only for a loop where no shape fits.
2. Spacer runs under a fixed budget counted in its own steps, or another deterministic bound, never a wall-clock timeout. Builds are reproducible and Precog never reads the clock, so the same build must prove the same sites every time.
3. Every invariant Spacer proposes is confirmed inductively by both Z3 and CVC5, as any proof is, before its sites count as proven, so a bug in Spacer cannot produce a wrong proof. CVC5 has no Horn engine, so it confirms rather than searches.
4. It is off by default, behind a switch only the measurement uses (a flag or environment variable of your choosing; name it in the report). Turning it on by default is the owner's decision after the numbers.
</design>

<measure>
Run each program with Spacer off and on, on the same machine with the same cores:
- examples/night_trains.cy first. Design claude's scratch build on 2026-10-25 left 12 of its 60 sites unproven; measure your own baseline, since the count may have moved.
- Then every example and kayo test program that has an unproven loop site with Spacer off.

For each program, record:
- unproven sites with Spacer off and on;
- for each site Spacer newly proves, its line and the invariant found;
- any invariant that failed confirmation, and where;
- Precog's time and the whole build's, off and on, as the median of three runs;
- the budget used, and how many loops hit it.
</measure>

<constraints>
- Cores: at most 3 at once (CLAUDE.md). std claude works only while you don't.
- Nothing is downloaded without the owner's yes. Spacer is already inside the linked Z3; if anything else turns out to be needed, stop and say what, through design claude.
- Keep to the measurement. Don't change the shapes, the 100 000-trip step budget or any other Precog behaviour, and don't turn Spacer on by default. If you find a bug in Precog along the way, note it in the report, and fix it only if the fix is small and leaves the baseline unchanged.
- The measuring script and its results are worth running again, so they go in the repo, inside kayo/ (as kayo/metal/bench/ does), not in a scratch folder.
- This is one connected task: work on it directly, without subagents.
- Finish the whole measurement before reporting. If something blocks it, report what blocks it and what you measured so far.
- Commit as usual: `kind.folder.narrowing: sentence` subjects, the `Co-Authored-By: compiler claude <noreply@anthropic.com>` trailer, `git pull --rebase` before pushing.
</constraints>

<report>
When it's done, report to design claude (local_9078e904-c5c3-41ca-a97b-29bc24a56ed6). Lead with the outcome in one sentence: how many sites Spacer proved, and at what build-time cost. Then give a table per program, then anything that surprised you. Give only figures you measured, and say how each was measured. The owner decides from this report whether Spacer stays, and with what budget.
</report>
