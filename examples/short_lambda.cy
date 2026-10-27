# A short lambda (owner, 2026-10-24: "(c)"): one value after `=`, its parameters' types and its return type taken from
# where it goes, `keep`'s `fn[i32] -> bool` or a declared type; captures stay explicit with `with`.
function keep [lent $xs: std::list[integer.signed.32_bit], $test: function[integer.signed.32_bit] -> boolean] -> std::list[integer.signed.32_bit]:
    variable $out: std::list[integer.signed.32_bit] = std::list.empty;
    for $x in lend $xs:
        if $test($x):
            std::list.append(lend.mutable $out, value: $x);
        end of if
    end of for
    return give $out;
end of function keep

!MAIN:
    let $scores: std::list[integer.signed.32_bit] = [55, 72, 60, 91, 38];
    let $limit := 60;
    let $passing := keep(lend $scores, function [$s] with [lend $limit] = $s at_least $limit);
    let $evens := keep(lend $scores, function [$s] = ($s modulo.floor 2) equal_to 0);
    std::print.shortest($passing, " ", $evens, newline);
    let $big: function[integer.signed.32_bit] -> boolean = function [$s] = $s at_least 90;
    std::print.shortest($big(91), " ", twice_if(5, function [$n] = $n at_most 9), newline);
!end of MAIN

function twice_if [$n: integer.signed.32_bit, $test: function[integer.signed.32_bit] -> boolean] -> integer.signed.32_bit:
    if $test($n):
        return $n times 2;
    end of if
    return $n;
end of function twice_if
