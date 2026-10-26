# Narrowed lists go where plain ones do and keep their kind (owner, 2026-10-25: "Seems good"): a page-aligned
# list grown by a function taking a plain list (its own version, which grows it page-aligned) and doubled on the
# GPU; a plain list made page-aligned by `to.list.page_aligned`; a not-page-aligned list under a plain name.
# 2 x 5001 = 10002, then 2 + 3 and 1 + 1 + 1: exit 10002 div 1000 + 5 + 3 = 18.
fn grow [lent.mut $xs: std::list[i32]] {
    for $i [from: 1, to: 5000, by: 1] { std::list.append(lend.mut $xs, value: 1); }
}
fn total [lent $xs: std::list[i32]] -> i32 {
    let mut $s: i32 = 0;
    for $x in lend $xs { change $s = $s + $x; }
    return $s;
}
MAIN {
    let mut $a: std::list.page_aligned[i32] = [1,];
    grow(lend.mut $a);
    let $k: i32 = 2;
    no_check inline.%MSL threads std::list.length(lend $a) {
        kernel void twice(device int *xs [[buffer(0)]], constant int &k [[buffer(1)]], uint id [[thread_position_in_grid]]) {
            xs[id] = xs[id] * k;
        }
    } with [lend.mut $a -> buffer(0), copy $k -> buffer(1)];
    let $plain: std::list[i32] = [2, 3];
    let $b := to.list.page_aligned(give $plain);
    let $na: std::list.not_page_aligned[i32] = std::list.fill(value: 1, count: 3);
    let $c: std::list[i32] = give $na;
    std::exit(code: total(lend $a) div.trunc 1000 + total(lend $b) + total(lend $c));
}
