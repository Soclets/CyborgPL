# `and.then` / `or.else` (owner, 2026-09-25): the right side is worked out only when the
# left has not already decided — "A, and then (only if A held) B", "A, or else (only if
# A failed) B". Plain `and` / `or` always work out both (examples/and_runs_both.cy).
proc loud [let $name: str.utf8, let $answer: bool] -> bool:
    std::print.shortest("checked ", $name, newln);
    ret $answer;
end of proc loud
!MAIN:
    let $xs := [3, 5, 8];
    let $i: int.unsign.64 := 7;
    # 'i' is past the end, so the list is never read
    if $i at_most std::ls.len(lend $xs) and.then $xs[pos: $i] eq 5:
        std::print.shortest("found", newln);
    end of if
    let $n: int.sign.32 := 0;
    if $n eq 0 or.else 100 div.floor $n lt 2:
        std::print.shortest("small or nothing", newln);
    end of if
    let $a := loud("first", false) and.then loud("second", true);
    let $b := loud("third", true) or.else loud("fourth", true);
    let $c := loud("fifth", true) and.then loud("sixth", false);
    std::print.shortest($a, " ", $b, " ", $c, newln);
!end of MAIN
