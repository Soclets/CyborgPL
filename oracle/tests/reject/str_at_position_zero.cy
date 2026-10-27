!MAIN:
    let $s := "hello";
    let $b := std::string.at.byte(lend $s, position: 0) otherwise 0;
!end of MAIN
