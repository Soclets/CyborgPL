!MAIN:
    let $limit := 60;
    let $f := fn [$s: int.sign.32] -> bool:
        ret $s at_least $limit;
    end of fn;
    let $b := $f(70);
!end of MAIN
