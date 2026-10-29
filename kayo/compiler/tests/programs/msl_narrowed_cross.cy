# Narrowed lists go where plain ones do and keep their kind (owner, 2026-10-25: "Seems good"): a page-aligned
# list grown by a function taking a plain list (its own version, which grows it page-aligned) and doubled on the
# GPU; a plain list made page-aligned by `to.list.page_aligned`; a not-page-aligned list under a plain name.
# 2 x 5001 = 10002, then 2 + 3 and 1 + 1 + 1: exit 10002 div 1000 + 5 + 3 = 18.
proc grow [lent.mut $xs: std::ls[i32]]:
    for $i [from: 1, to: 5000, by: 1]:
        std::ls.append(lend.mut $xs, value: 1);
    ;for
;proc grow
fn total [lent $xs: std::ls[i32]] -> i32:
    var $s: i32 = 0;
    for $x in lend $xs:
        change $s to $s + $x;
    ;for
    ret $s;
;fn total
MAIN:
    var $a: std::ls.page_aligned[i32] = [1,];
    grow(lend.mut $a);
    let $k: i32 := 2;
    no_check inline.%MSL threads std::ls.len(lend $a) with [lend.mut $a -> buffer(0), copy $k -> buffer(1)]:
        kernel void twice(device int *xs [[buffer(0)]], constant int &k [[buffer(1)]], uint id [[thread_position_in_grid]]) {
            xs[id] = xs[id] * k;
        }
    ;inline.%MSL
    let $plain: std::ls[i32] := [2, 3];
    let $b := to.ls.page_aligned(give $plain);
    let $na: std::ls.not_page_aligned[i32] := std::ls.fill(value: 1, count: 3);
    let $c: std::ls[i32] := give $na;
    std::exit(code: total(lend $a) div.trunc 1000 + total(lend $b) + total(lend $c));
;MAIN
