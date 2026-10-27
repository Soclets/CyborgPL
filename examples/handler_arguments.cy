# A `coll.handler` member taking arguments (owner, 2026-10-23: "Yes"): defined in `[ ]`, given in `( )` with their
# words, always through the collection's name, since `otherwise f(…)` alone is a fallback value from a call.
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
    half_failed.or_default [$default: integer.signed.32_bit, $label: string.utf8]:
        !case odd:
            std::print.shortest($label, ": odd, using ", $default, newline); return $default;
        !end of case odd
        !case negative $v:
            std::print.shortest($label, ": ", $v, " is negative, using ", $default, newline); return $default;
        !end of case negative
    end of half_failed.or_default
end of collection.handler half_failed

procedure half_or [$n: integer.signed.32_bit, $d: integer.signed.32_bit] -> integer.signed.32_bit:
    let $h := half($n) otherwise half_failed.!or_default(default: $d, label: "half_or");
    return $h;
end of procedure half_or

!MAIN:
    std::print.shortest(half_or(8, 0), " ", half_or(7, 100), " ", half_or(negative 4, 5 times 2), newline);
!end of MAIN
