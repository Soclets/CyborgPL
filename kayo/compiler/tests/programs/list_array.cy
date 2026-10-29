# `to.array[…]` freezes a list's length, `to.list[…]` lets an array grow
# (owner, 2026-09-23). `give` hands the elements over without a copy;
# `copy` keeps the original. Texts inside are copied, not shared: the
# leak check frees each once.
%extern.C proc exit [$code: i32];
MAIN:
    var $l: std::ls[i32] = [1, 2];
    std::ls.append(lend.mut $l, value: 3);
    let $a := to.arr(give $l);
    var $m = to.ls(copy $a);
    std::ls.append(lend.mut $m, value: 4);
    var $s: std::ls[i32] = arr.slice(lend $a, from: pos 2, to: pos 3);
    std::ls.append(lend.mut $s, value: 9);
    var $w: std::ls[str.utf8] = ["x", "yy"];
    std::ls.append(lend.mut $w, value: "zzz");
    let $fw := to.arr(copy $w);
    let $back := to.ls(give $fw);
    # 3 + 4 x 10 + 3 x 100 + 3 + (1 + 4 + 9)
    let $n: u64 := arr.len(lend $a) + std::ls.len(lend $m) x 10 + std::ls.len(lend $s) x 100 + std::ls.len(lend $back);
    let $v: i32 := $a[pos: 1] + $m[pos: 4] + $s[pos: 3];
    no_check my.exit(to.i32($n) + $v);
;MAIN
