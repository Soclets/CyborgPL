!MAIN:
    let $s := "abc";
    let $t := std::string.slice.byte(lend $s, from: position 1, to: position 2) otherwise string.empty;
!end of MAIN
