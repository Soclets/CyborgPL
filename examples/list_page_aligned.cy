# A list's kinds (owner, 2026-10-25): `std::list.page_aligned[T]` always starts on a page, so a GPU block takes it
# without a copy; `std::list.not_page_aligned[T]` never does; a plain `std::list[T]` is Precog's to place. A narrowed
# list goes anywhere a plain one does, lent or given, and keeps its kind as it grows there; a plain list becomes one
# only by `to.list.page_aligned(…)` / `to.list.not_page_aligned(…)`, which may copy it.
fn total [lent $xs: std::ls[int.sign.32]] -> int.sign.32:
    var $s: int.sign.32 = 0;
    for $x in lend $xs:
        change $s = $s plus $x;
    end of for
    ret $s;
end of fn total

proc grow [lent.mut $xs: std::ls[int.sign.32]]:
    std::ls.append(lend.mut $xs, value: 4);
end of proc grow

!MAIN:
    var $xs: std::ls.page_aligned[int.sign.32] = [1, 2, 3];
    grow(lend.mut $xs);
    std::print.shortest(total(lend $xs), " ", $xs, newln);
    let $plain: std::ls[int.sign.32] = [5, 6];
    let $pa := to.ls.page_aligned(give $plain);
    let $np: std::ls.not_page_aligned[int.sign.32] = to.ls.not_page_aligned(give $pa);
    let $back: std::ls[int.sign.32] = give $np;
    std::print.shortest($back, newln);
!end of MAIN
