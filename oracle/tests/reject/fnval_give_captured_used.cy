!MAIN:
    let $s := "text";
    let $f := fn [] -> int.sign.32 with [give $s]:
        ret std::str.bytes(lend $s);
    end of fn;
    std::print.stored($s);
!end of MAIN
