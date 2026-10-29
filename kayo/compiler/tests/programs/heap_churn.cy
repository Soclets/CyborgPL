# Built and discarded over and over, inside a call, so anything not given
# back becomes unreachable — which is what a leak checker can see. A
# handle still sitting on the stack at exit looks reachable and would be
# reported as no leak at all.
%extern.C proc exit [let $code: i32];

fn round [let $n: i32] -> i32:
    var $xs: std::ls[i32] = [];
    for $i: i32 [from: 1, to: 20, by: 1]:
        std::ls.append(lend.mut $xs, value: $i + $n);
    ;for
    var $text: str.utf8 = str.empty;
    for $j: i32 [from: 1, to: 5, by: 1]:
        change $text to copy $text concat "ab";
    ;for
    var $m: std::map[str.utf8, std::ls[i32]] = std::map.empty;
    std::map.set(lend.mut $m, key: copy $text, value: copy $xs);
    std::map.set(lend.mut $m, key: "other", value: copy $xs);
    let $entries: i32 := std::map.count(lend $m);
    let $bytes: i32 := std::str.bytes(lend $text);
    let $count: i32 := std::ls.len(lend $xs);
    ret $entries + $bytes + $count;
;fn round

MAIN:
    var $total: i32 = 0;
    for $k: i32 [from: 1, to: 500, by: 1]:
        change $total to my.round($k);
    ;for
    no_check my.exit($total - 32);
;MAIN
