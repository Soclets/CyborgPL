!MAIN:
    let $p: str.utf8 := "ab";
    let $b := std::str.contains("abc", part: copy $p);
!end of MAIN
