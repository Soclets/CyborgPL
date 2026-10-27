# Functions that take any number of arguments, typed or not.
fn sum [...$nums] -> float.bin.64:
    var $total: float.bin.64 = 0;
    for $n in lend $nums:
        change $total = $total plus to.float.bin.64($n);
    end of for
    ret $total;
end of fn sum

fn count_ints [...$nums: int.sign.32] -> int.unsign.64:
    ret std::ls.len(lend $nums);
end of fn count_ints

proc show [lent $label: str.utf8, lent ...$parts]:
    std::print.shortest($label, ": ");
    for $p in lend $parts:
        std::print.shortest($p, " ");
    end of for
    std::print.stored(newln);
end of proc show

!MAIN:
    std::print.shortest(sum(1, 2.5, 3), newln);
    std::print.shortest(sum(), newln);
    std::print.shortest(count_ints(4, 5, 6), newln);
    let $name := "Ann";
    show("mixed", lend $name, 7, true, `x`);
    show("none");
!end of MAIN
