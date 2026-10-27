!MAIN:
    let $p: string.utf8 = "ab";
    let $b := std::string.contains("abc", part: copy $p);
!end of MAIN
