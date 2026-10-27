function half [$n: integer.signed.32_bit] -> integer.signed.32_bit or fail[odd, negative: integer.signed.32_bit]:
    if $n less_than 0:
        fail negative[$n];
    end of if
    return $n;
end of function half
collection.handler h:
    h.zero:
        !case odd:
            return 0;
        !end of case odd
        !case negative $v:
            return 0;
        !end of case negative
    end of h.zero
end of collection.handler h
function f [] -> integer.signed.32_bit:
    let $v := half(3) otherwise h.nope;
    return $v;
end of function f
!MAIN:
!end of MAIN
