# `list.remove` hands the removed element back (decided 2026-09-21): no copy, no hole.

rec Job:
    var name: str.utf8;
end of rec Job
proc run [let $job: Job]:
    std::print.shortest("ran ", $job.name, newln);
end of proc run
!MAIN:
    var $queue = [Job[name = "a"], Job[name = "b"], Job[name = "c"]];
    run(std::ls.remove(lend.mut $queue, pos: 1));              # straight into a call
    let $next := std::ls.remove(lend.mut $queue, pos: 1);     # or kept
    run(give $next);
    std::ls.remove(lend.mut $queue, pos: 1);                   # or dropped
    std::print.shortest(std::ls.len(lend $queue), newln);
!end of MAIN
