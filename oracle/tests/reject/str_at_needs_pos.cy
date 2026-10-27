!MAIN:
    let $s := "abc";
    let $c := std::string.at.code_point(lend $s, 1) otherwise `?`;
!end of MAIN
