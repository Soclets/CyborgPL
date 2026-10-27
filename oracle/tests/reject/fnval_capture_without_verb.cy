!MAIN:
    let $limit := 60;
    let $f := fn [$s: int.sign.32] -> bool with [$limit]:
        ret true;
    end of fn;
!end of MAIN
