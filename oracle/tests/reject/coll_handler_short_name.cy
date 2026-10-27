fn half [$n: int.sign.32] -> int.sign.32 or fail[odd]:
    if $n times 1 eq 3:
        fail odd;
    end of if
    ret $n;
end of fn half

coll.handler h:
    as_zero:
        !case odd:
            ret 0;
        !end of case odd
    end of as_zero
end of coll.handler h

!MAIN:
!end of MAIN
