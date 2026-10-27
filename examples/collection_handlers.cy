# `coll.handler` (owner, 2026-10-13): named `otherwise` blocks. Each member holds the arms written
# after `otherwise` today, one per way the call can fail, and `otherwise half_failed.as_zero` reads them
# as if they were written at the call: an arm may `return`, `break`, `continue` or `fail` from where it
# is used, and a member that misses a way the call can fail is refused there.
fn half [let $n: int.sign.32] -> int.sign.32 or fail[odd, negative: int.sign.32]:
    if $n less_than 0:
        fail negative[$n];
    end of if
    if ($n div.floor 2) times 2 not_eq $n:
        fail odd;
    end of if
    ret $n div.floor 2;
end of fn half

coll.handler half_failed:
    member half_failed.as_zero:
        !case odd:
            ret 0;
        !end of case odd
        !case negative $v:
            std::print.shortest("negative: ", $v, newln); ret 0;
        !end of case negative
    end of member half_failed.as_zero
    member half_failed.skip:
        !case odd:
            continue;
        !end of case odd
        !case negative $v:
            continue;
        !end of case negative
    end of member half_failed.skip
    member half_failed.stop:
        !case odd:
            break;
        !end of case odd
        !case negative $v:
            break;
        !end of case negative
    end of member half_failed.stop
end of coll.handler half_failed

proc half_or_zero [let $n: int.sign.32] -> int.sign.32:
    let $h := half($n) otherwise half_failed.!as_zero;
    ret $h;
end of proc half_or_zero

fn sum_of_halves [lent $xs: std::ls[int.sign.32], let $until_bad: bool] -> int.sign.32:
    var $sum := 0;
    for $x in lend $xs:
        if $until_bad:
            let $h := half($x) otherwise half_failed.!stop;
            change $sum = $sum plus $h;
        end of if
        else:
            let $h := half($x) otherwise half_failed.!skip;
            change $sum = $sum plus $h;
        end of else
    end of for
    ret $sum;
end of fn sum_of_halves

!MAIN:
    std::print.shortest(half_or_zero(10), " ", half_or_zero(7), newln);
    std::print.shortest(half_or_zero(neg 4), newln);
    let $xs: std::ls[int.sign.32] = [2, 4, 5, 8, neg 2, 10];
    std::print.shortest(sum_of_halves(lend $xs, false), " ", sum_of_halves(lend $xs, true), newln);
!end of MAIN
