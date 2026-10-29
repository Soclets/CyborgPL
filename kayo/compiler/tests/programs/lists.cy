%extern.C proc exit [$code: i32];
MAIN:
    var $xs: std::ls[i32] = [10, 20, 30];
    std::ls.append(lend.mut $xs, value: 12);
    let $n: i32 := std::ls.len(lend $xs);
    let $a: i32 := $xs[pos: 1];
    let $d: i32 := $xs[pos: 4];
    no_check my.exit($n + $a + $d);
;MAIN
