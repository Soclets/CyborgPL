MAIN:
    let $s := "hello";
    let $b := std::str.at.byte(lend $s, pos: 0) ?? 0;
;MAIN
