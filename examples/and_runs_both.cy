# `and` and `or` always work out both sides, left then right: nothing written on the
# line is skipped (owner, 2026-09-25: b). A check that protects the next step is
# therefore written as two `if`s; tests/reject/and_does_not_guard.cy is the one-line
# form, which stops when the right side is out of range.
procedure loud [$name: string.utf8, $answer: boolean] -> boolean:
    std::print.shortest("checked ", $name, newline);
    return $answer;
end of procedure loud
!MAIN:
    let $ok := loud("first", false) and loud("second", true);    # both print
    std::print.shortest($ok, newline);
    let $xs := [3, 5, 8];
    let $i: integer.unsigned.64_bit = 7;
    if $i at_most std::list.length(lend $xs):
        if $xs[position: $i] equal_to 5:
            std::print.shortest("found", newline);
        end of if
    end of if
    std::print.shortest("done", newline);
!end of MAIN
