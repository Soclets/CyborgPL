# `and` and `or` always work out both sides, left then right: nothing written on the
# line is skipped (owner, 2026-09-25: b). A check that protects the next step is
# therefore written as two `if`s; tests/reject/and_does_not_guard.cy is the one-line
# form, which stops when the right side is out of range.
proc loud [let $name: str.utf8, let $answer: bool] -> bool:
    std::print("checked ", $name, newln);
    ret $answer;
;proc loud
MAIN:
    let $ok := loud("first", false) and loud("second", true);    # both print
    std::print($ok, newln);
    let $xs := [3, 5, 8];
    let $i: u64 := 7;
    if $i <= std::ls.len(lend $xs):
        if $xs[pos: $i] == 5:
            std::print("found", newln);
        ;if
    ;if
    std::print("done", newln);
;MAIN
