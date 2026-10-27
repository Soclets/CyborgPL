# `list.remove` hands the removed element back (decided 2026-09-21): no copy, no hole.

record Job:
    name: string.utf8;
end of record Job
procedure run [$job: Job]:
    std::print.shortest("ran ", $job.name, newline);
end of procedure run
!MAIN:
    variable $queue := [Job[name = "a"], Job[name = "b"], Job[name = "c"]];
    run(std::list.remove(lend.mutable $queue, position: 1));              # straight into a call
    let $next := std::list.remove(lend.mutable $queue, position: 1);     # or kept
    run(give $next);
    std::list.remove(lend.mutable $queue, position: 1);                   # or dropped
    std::print.shortest(std::list.length(lend $queue), newline);
!end of MAIN
