%extern.C proc exit [$code: i32];
proc fill [lent.mut $into: std::ls[i32]]:
    std::ls.append(lend.mut $into, value: 7);
;proc fill
MAIN:
    var $b: std::ls[i32] = [];
    fill(lend.mut $b);
    let $first: i32 := $b[pos: 1];
    no_check my.exit($first);
;MAIN
