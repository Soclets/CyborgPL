# `list.remove` hands the removed element back (decided 2026-09-21): no copy, no hole.

struct Job { name: str.utf8 }
fn run [$job: Job] { std::print.useful("ran ", $job.name, newln); }
MAIN {
    let mut $queue := [Job[name = "a"], Job[name = "b"], Job[name = "c"]];
    run(std::list.remove(lend.mut $queue, pos: 1));              # straight into a call
    let $next := std::list.remove(lend.mut $queue, pos: 1);     # or kept
    run(give $next);
    std::list.remove(lend.mut $queue, pos: 1);                   # or dropped
    std::print.useful(std::list.length(lend $queue), newln);
}
