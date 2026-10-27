# Functions that take any number of arguments, typed or not.
function sum [...$nums] -> float.binary.64_bit:
    variable $total: float.binary.64_bit = 0;
    for $n in lend $nums:
        change $total = $total plus to.float.binary.64_bit($n);
    end of for
    return $total;
end of function sum

function count_ints [...$nums: integer.signed.32_bit] -> integer.unsigned.64_bit:
    return std::list.length(lend $nums);
end of function count_ints

procedure show [lent $label: string.utf8, lent ...$parts]:
    std::print.shortest($label, ": ");
    for $p in lend $parts:
        std::print.shortest($p, " ");
    end of for
    std::print.stored(newline);
end of procedure show

!MAIN:
    std::print.shortest(sum(1, 2.5, 3), newline);
    std::print.shortest(sum(), newline);
    std::print.shortest(count_ints(4, 5, 6), newline);
    let $name := "Ann";
    show("mixed", lend $name, 7, true, `x`);
    show("none");
!end of MAIN
