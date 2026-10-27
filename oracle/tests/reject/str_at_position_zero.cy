!MAIN:
    let $s := "hello";
    let $b := std::str.at.byte(lend $s, pos: 0) otherwise 0;
!end of MAIN
