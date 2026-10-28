# A list's kinds (owner, 2026-10-25): `std::list.page_aligned[T]` always starts on a page, so a GPU block takes it
# without a copy; `std::list.not_page_aligned[T]` never does; a plain `std::list[T]` is Precog's to place. A narrowed
# list goes anywhere a plain one does, lent or given, and keeps its kind as it grows there; a plain list becomes one
# only by `to.list.page_aligned(…)` / `to.list.not_page_aligned(…)`, which may copy it.
fn total [lent $xs: std::ls[i32]] -> i32:
    var $s: i32 = 0;
    for $x in lend $xs:
        change $s += $x;
    ;for
    ret $s;
;fn total

proc grow [lent.mut $xs: std::ls[i32]]:
    std::ls.append(lend.mut $xs, value: 4);
;proc grow

MAIN:
    var $xs: std::ls.page_aligned[i32] = [1, 2, 3];
    grow(lend.mut $xs);
    std::println(total(lend $xs), " ", $xs);
    let $plain: std::ls[i32] := [5, 6];
    let $pa := to.ls.page_aligned(give $plain);
    let $np: std::ls.not_page_aligned[i32] := to.ls.not_page_aligned(give $pa);
    let $back: std::ls[i32] := give $np;
    std::println($back);
;MAIN
