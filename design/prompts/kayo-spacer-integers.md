The owner gives the go: measure Spacer with integer clauses, the second way you named in your Spacer report. This is your normal work as compiler claude, under design/compiler-session.md and CLAUDE.md, which still apply in full.

<context>
Your Spacer report (1db30d4b, 631aba23; kayo/tools/spacer_probe/MEASURED.md and RESULTS.md) found Spacer proved 1 of the 40 sites left unproven behind a loop, for +11.3 s of Precog time over the 22 programs that asked it. 41 of the 52 questions were refuted, mostly sites that really can fail. 6 spent the 10,000,000-step budget, and 100,000,000 steps answered none of them. One of the 6 looks provable: in msl_whole_pages, `$wrong + 1` never passes the list's 16,384 elements. Your hand probe of 2026-10-25 had integer clauses answered in 0.03 s where bit-vector ones ran out, and you suggested integer clauses with the guess confirmed in bit-vectors as the other way to try. The owner chose to test that before deciding whether Spacer stays and with what budget.
</context>

<design>
1. Spacer searches over integer clauses: each machine value read as a mathematical integer, with the bounds of its type (`0 ≤ $k < 2^64` for a u64, and so on) stated as facts.
2. Each invariant it proposes is translated back and confirmed in bit-vectors, by Z3 and CVC5 both, exactly as the current path confirms one (it holds on every way into the loop, and one trip round keeps it), before the site is asked again with it assumed. A guess that only holds over the integers is rejected there, so nothing can be proven that the machine's arithmetic breaks.
3. The same deterministic budget as now, Z3's rlimit, never a clock.
4. Keep the current bit-vector path. Run the integer path as a second mode beside it, chosen by a switch you name in the report, so both can be measured on the same build. The shapes run first and Spacer only where they leave a site unproven, as now. The default stays as it is, bit-vector Spacer on, until the owner decides.
</design>

<measure>
On the 22 programs that asked Spacer anything, plus your two-counter scratch program if you keep it in kayo/tools/spacer_probe/, run three modes: Spacer off, bit-vector Spacer (the current default), integer Spacer. Use the same machine and cores, and take the median of three runs. For each program, record:
- unproven sites in each mode;
- each site newly proven, with its line and the invariant;
- every invariant the integer path proposed that bit-vector confirmation rejected, with the reason if you can tell;
- Precog's time and the whole build's;
- how many questions spent the budget.
Say for msl_whole_pages in particular whether the integer path proves `$wrong + 1`.
</measure>

<constraints>
- Cores: at most 3 at once (CLAUDE.md). std claude works only while you don't; tell design claude when you start and finish, so std claude can be told.
- Nothing is downloaded without the owner's yes; ask through design claude if anything new is needed.
- Keep to this measurement. Don't change the shapes, the 100 000-trip budget, the bit-vector path's behaviour or the default.
- The script and results go in kayo/tools/spacer_probe/ beside the first measurement, not in a scratch folder.
- This is one connected task: work on it directly, without subagents.
- Finish the whole measurement before reporting. If something blocks it, report what blocks it and what you measured so far.
- Commit as usual: `kind.folder.narrowing: sentence` subjects, the `Co-Authored-By: compiler claude <noreply@anthropic.com>` trailer, `git pull --rebase` before pushing.
</constraints>

<report>
When it's done, report to design claude (local_9078e904-c5c3-41ca-a97b-29bc24a56ed6). Lead with the outcome in one sentence: how many sites the integer path proved beyond bit-vector Spacer, and at what cost. Then give a table per program for the three modes, then anything that surprised you. Give only figures you measured, and say how. The owner decides from this report whether Spacer stays, in which form, and with what budget.
</report>
