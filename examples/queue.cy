# `std::ls.remove` hands the removed element back (decided 2026-09-21): no copy, no hole.

rec Job:
    var name: str.utf8;
;rec Job
proc run [$job: Job]:
    std::print("ran ", $job.name, newln);
;proc run
MAIN:
    var $queue = [Job[name = "a"], Job[name = "b"], Job[name = "c"]];
    run(std::ls.remove(lend.mut $queue, pos: 1));              # straight into a call
    let $next := std::ls.remove(lend.mut $queue, pos: 1);     # or kept
    run(give $next);
    std::ls.remove(lend.mut $queue, pos: 1);                   # or dropped
    std::print(std::ls.len(lend $queue), newln);
;MAIN
