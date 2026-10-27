!MAIN:
    let $s := "hello";
    let $i: int.sign.64 := 0;
    let $b := std::str.at.byte(lend $s, pos: $i) otherwise 0;
    std::print.shortest($b, newln);
!end of MAIN
