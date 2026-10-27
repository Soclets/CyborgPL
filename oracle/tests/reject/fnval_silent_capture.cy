!MAIN:
    let $limit := 60;
    let $f := function [$s: integer.signed.32_bit] -> boolean:
        return $s at_least $limit;
    end of function;
    let $b := $f(70);
!end of MAIN
