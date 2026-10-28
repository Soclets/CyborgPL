# check claude's borrow findings (2026-09-29): a borrow never aliases a change and never outlives its owner (core-features.md)
MAIN:
    var $s: std::ls[i32] = [1, 2, 3];
    let $r := lend $s;
    change $s[pos: 1] to 99;
    std::print($r[pos: 1], newln);
;MAIN
