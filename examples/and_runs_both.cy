# `and` and `or` always work out both sides, left then right: nothing written on the
# line is skipped (owner, 2026-09-25: b). A check that protects the next step is
# therefore written as two `if`s; tests/reject/and_does_not_guard.cy is the one-line
# form, which stops when the right side is out of range.
proc loud [let $name: str.utf8, let $answer: bool] -> bool:
    std::print.shortest("checked ", $name, newln);
    ret $answer;
end of proc loud
!MAIN:
    let $ok := loud("first", false) and loud("second", true);    # both print
    std::print.shortest($ok, newln);
    let $xs := [3, 5, 8];
    let $i: int.unsign.64 = 7;
    if $i at_most std::ls.len(lend $xs):
        if $xs[pos: $i] eq 5:
            std::print.shortest("found", newln);
        end of if
    end of if
    std::print.shortest("done", newln);
!end of MAIN
