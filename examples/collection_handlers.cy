# `coll.handler` (owner, 2026-10-13): named `otherwise` blocks. Each member holds the arms written
# after `otherwise` today, one per way the call can fail, and `otherwise half_failed.as_zero` reads them
# as if they were written at the call: an arm may `return`, `break`, `continue` or `fail` from where it
# is used, and a member that misses a way the call can fail is refused there.
function half [$n: integer.signed.32_bit] -> integer.signed.32_bit or fail[odd, negative: integer.signed.32_bit]:
    if $n less_than 0:
        fail negative[$n];
    end of if
    if ($n divide.floor 2) times 2 not_equal_to $n:
        fail odd;
    end of if
    return $n divide.floor 2;
end of function half

collection.handler half_failed:
    half_failed.as_zero:
        !case odd:
            return 0;
        !end of case odd
        !case negative $v:
            std::print.shortest("negative: ", $v, newline); return 0;
        !end of case negative
    end of half_failed.as_zero
    half_failed.skip:
        !case odd:
            continue;
        !end of case odd
        !case negative $v:
            continue;
        !end of case negative
    end of half_failed.skip
    half_failed.stop:
        !case odd:
            break;
        !end of case odd
        !case negative $v:
            break;
        !end of case negative
    end of half_failed.stop
end of collection.handler half_failed

procedure half_or_zero [$n: integer.signed.32_bit] -> integer.signed.32_bit:
    let $h := half($n) otherwise half_failed.!as_zero;
    return $h;
end of procedure half_or_zero

function sum_of_halves [lent $xs: std::list[integer.signed.32_bit], $until_bad: boolean] -> integer.signed.32_bit:
    variable $sum := 0;
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
    return $sum;
end of function sum_of_halves

!MAIN:
    std::print.shortest(half_or_zero(10), " ", half_or_zero(7), newline);
    std::print.shortest(half_or_zero(negative 4), newline);
    let $xs: std::list[integer.signed.32_bit] = [2, 4, 5, 8, negative 2, 10];
    std::print.shortest(sum_of_halves(lend $xs, false), " ", sum_of_halves(lend $xs, true), newline);
!end of MAIN
