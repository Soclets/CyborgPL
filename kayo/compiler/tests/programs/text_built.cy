%extern.C proc exit [$code: i32];
MAIN:
    var $out: str.utf8 = str.empty;
    for $i: i32 [from: 1, to: 20, by: 1]:
        change $out to copy $out concat "ab";
    ;for
    let $n: i32 := std::str.bytes(lend $out);
    let $lit: str.utf8 := "fixed";
    let $dup: str.utf8 := copy $lit;
    let $m: i32 := std::str.bytes(lend $dup);
    no_check my.exit($n + $m);
;MAIN
