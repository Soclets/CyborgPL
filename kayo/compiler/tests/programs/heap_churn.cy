# Built and discarded over and over, inside a call, so anything not given
# back becomes unreachable — which is what a leak checker can see. A
# handle still sitting on the stack at exit looks reachable and would be
# reported as no leak at all.
extern.C fn exit [$code: i32];

fn round [$n: i32] -> i32 {
    let mut $xs: std::list[i32] = [];
    for $i: i32 [from: 1, to: 20, by: 1] {
        std::list.append(lend.mut $xs, value: $i plus $n);
    }
    let mut $text: str.utf8 = str.empty;
    for $j: i32 [from: 1, to: 5, by: 1] {
        change $text = copy $text concat "ab";
    }
    let mut $m: std::map[str.utf8, std::list[i32]] = std::map.empty;
    std::map.set(lend.mut $m, key: copy $text, value: copy $xs);
    std::map.set(lend.mut $m, key: "other", value: copy $xs);
    let $entries: i32 = std::map.length(lend $m);
    let $bytes: i32 = std::str.bytes(lend $text);
    let $count: i32 = std::list.length(lend $xs);
    return $entries plus $bytes plus $count;
}

MAIN {
    let mut $total: i32 = 0;
    for $k: i32 [from: 1, to: 500, by: 1] {
        change $total = my.round($k);
    }
    no_check my.exit($total minus 32);
}
