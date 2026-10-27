!MAIN:
    let $limit := 60;
    let $f := function [$s: integer.signed.32_bit] -> boolean with [$limit]:
        return true;
    end of function;
!end of MAIN
