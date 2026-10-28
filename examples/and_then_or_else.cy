# `and.then` / `or.else` (owner, 2026-09-25): the right side is worked out only when the
# left has not already decided — "A, and then (only if A held) B", "A, or else (only if
# A failed) B". Plain `and` / `or` always work out both (examples/and_runs_both.cy).
proc loud [let $name: str.utf8, let $answer: bool] -> bool:
    std::println("checked ", $name);
    ret $answer;
;proc loud
MAIN:
    let $xs := [3, 5, 8];
    let $i: u64 := 7;
    # 'i' is past the end, so the list is never read
    if $i <= std::ls.len(lend $xs) && $xs[pos: $i] == 5:
        std::println("found");
    ;if
    let $n: i32 := 0;
    if $n == 0 || 100 div.floor $n < 2:
        std::println("small or nothing");
    ;if
    let $a := loud("first", false) && loud("second", true);
    let $b := loud("third", true) || loud("fourth", true);
    let $c := loud("fifth", true) && loud("sixth", false);
    std::println($a, " ", $b, " ", $c);
;MAIN
