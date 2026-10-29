%extern.C proc exit [$code: i32];
MAIN:
    let $a: std::ls[i32] := [1, 2, 3];
    var $b: std::ls[i32] = copy $a;
    std::ls.append(lend.mut $b, value: 4);
    change $b[pos: 1] to 9;
    let $la: i32 := std::ls.len(lend $a);
    let $lb: i32 := std::ls.len(lend $b);
    let $a1: i32 := $a[pos: 1];
    let $b1: i32 := $b[pos: 1];
    no_check my.exit($la x 1 + $lb x 10 + $a1 x 100 + $b1 x 1000);
;MAIN
