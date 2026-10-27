!MAIN:
    let $s := "abc";
    let $t := std::str.slice.byte(lend $s, from: pos 1, to: pos 2) otherwise str.empty;
!end of MAIN
