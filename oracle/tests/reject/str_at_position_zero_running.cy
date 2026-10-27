!MAIN:
    let $s := "hello";
    let $i: integer.signed.64_bit = 0;
    let $b := std::string.at.byte(lend $s, position: $i) otherwise 0;
    std::print.shortest($b, newline);
!end of MAIN
