# `and.then` / `or.else` (owner, 2026-09-25): the right side is worked out only when the
# left has not already decided — "A, and then (only if A held) B", "A, or else (only if
# A failed) B". Plain `and` / `or` always work out both (examples/and_runs_both.cy).
procedure loud [$name: string.utf8, $answer: boolean] -> boolean:
    std::print.shortest("checked ", $name, newline);
    return $answer;
end of procedure loud
!MAIN:
    let $xs := [3, 5, 8];
    let $i: integer.unsigned.64_bit = 7;
    # 'i' is past the end, so the list is never read
    if $i at_most std::list.length(lend $xs) and.then $xs[position: $i] equal_to 5:
        std::print.shortest("found", newline);
    end of if
    let $n: integer.signed.32_bit = 0;
    if $n equal_to 0 or.else 100 divide.floor $n less_than 2:
        std::print.shortest("small or nothing", newline);
    end of if
    let $a := loud("first", false) and.then loud("second", true);
    let $b := loud("third", true) or.else loud("fourth", true);
    let $c := loud("fifth", true) and.then loud("sixth", false);
    std::print.shortest($a, " ", $b, " ", $c, newline);
!end of MAIN
