# `list.remove` in range, its position computed (check claude, 2026-10-08):
# 7, then 5; two left, then one: 7 + 5 x 10 + 1 x 100 = 157.
%extern.C fn exit [$code: i32];
fn three [] -> u64 { return 3; }
MAIN {
    let mut $xs: std::list[i32] = [5, 6, 7];
    let $k := three();
    let $a := std::list.remove(lend.mut $xs, pos: $k);
    let $b := std::list.remove(lend.mut $xs, pos: $k minus 2);
    no_check my.exit($a plus $b times 10 plus to.i32(std::list.length(lend $xs)) times 100);
}
