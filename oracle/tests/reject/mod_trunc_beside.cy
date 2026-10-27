!MAIN:
    let $a: int.sign.32 := neg 7;
    std::print.shortest($a mod.trunc 2 plus 1, newln);
!end of MAIN
