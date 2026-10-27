function half [$n: integer.signed.32_bit] -> integer.signed.32_bit or fail[odd]:
    if $n times 1 equal_to 3:
        fail odd;
    end of if
    return $n;
end of function half

collection.handler h:
    as_zero:
        !case odd:
            return 0;
        !end of case odd
    end of as_zero
end of collection.handler h

!MAIN:
!end of MAIN
