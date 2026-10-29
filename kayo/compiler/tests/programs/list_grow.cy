%extern.C proc exit [let $code: i32];
MAIN:
    var $xs: std::ls[i32] = [];
    for $i: i32 [from: 1, to: 100, by: 1]:
        std::ls.append(lend.mut $xs, value: $i);
    ;for
    let $len: i32 := std::ls.len(lend $xs);
    let $last: i32 := $xs[pos: 100];
    let $first: i32 := std::ls.remove(lend.mut $xs, pos: 1);
    let $now_first: i32 := $xs[pos: 1];
    let $shorter: i32 := std::ls.len(lend $xs);
    no_check my.exit($len - $last + $first + $now_first + $shorter - 99);
;MAIN
