fn half [$n: int.sign.32] -> int.sign.32 or fail[odd, negative: int.sign.32]:
    if $n less_than 0:
        fail negative[$n];
    end of if
    if ($n div.floor 2) times 2 not_eq $n:
        fail odd;
    end of if
    ret $n div.floor 2;
end of fn half

coll.handler half_failed:
    half_failed.or_default [$default: int.sign.32, $label: str.utf8]:
        !case odd:
            std::print.shortest($label, ": odd, using ", $default, newln); ret $default;
        !end of case odd
        !case negative $v:
            std::print.shortest($label, ": ", $v, " is negative, using ", $default, newln); ret $default;
        !end of case negative
    end of half_failed.or_default
end of coll.handler half_failed

fn half_or [$n: int.sign.32, $d: int.sign.32] -> int.sign.32:
    let $h := half($n) otherwise half_failed.!or_default($d, "x");
    ret $h;
end of fn half_or

!MAIN:
    std::print.shortest(half_or(8, 0), " ", half_or(7, 100), " ", half_or(neg 4, 5 times 2), newln);
!end of MAIN
