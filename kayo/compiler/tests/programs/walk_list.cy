%extern.C proc exit [let $code: i32];
MAIN:
    let $xs: std::ls[i32] := [4, 5, 6];
    var $acc: i32 = 0;
    for $x in lend $xs:
        change $acc to $acc + $x;
    ;for
    no_check my.exit($acc);
;MAIN
