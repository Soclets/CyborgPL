# A `coll.handler` member taking arguments (owner, 2026-10-23: "Yes"): defined in `[ ]`, given in `( )` with their
# words, always through the collection's name, since `otherwise f(…)` alone is a fallback value from a call.
fn half [let $n: int.sign.32] -> int.sign.32 or fail[odd, negative: int.sign.32]:
    if $n lt 0:
        fail negative[$n];
    end of if
    if ($n div.floor 2) times 2 not_eq $n:
        fail odd;
    end of if
    ret $n div.floor 2;
end of fn half

coll.handler half_failed:
    member half_failed.or_default [let $default: int.sign.32, let $label: str.utf8]:
        !case odd:
            std::print.shortest($label, ": odd, using ", $default, newln); ret $default;
        !end of case odd
        !case negative $v:
            std::print.shortest($label, ": ", $v, " is negative, using ", $default, newln); ret $default;
        !end of case negative
    end of member half_failed.or_default
end of coll.handler half_failed

proc half_or [let $n: int.sign.32, let $d: int.sign.32] -> int.sign.32:
    let $h := half($n) otherwise half_failed.!or_default(default: $d, label: "half_or");
    ret $h;
end of proc half_or

!MAIN:
    std::print.shortest(half_or(8, 0), " ", half_or(7, 100), " ", half_or(neg 4, 5 times 2), newln);
!end of MAIN
