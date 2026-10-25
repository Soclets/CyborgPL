# A list's kinds (owner, 2026-10-25): `std::list.page_aligned[T]` always starts on a page, so a GPU block takes it
# without a copy; `std::list.not_page_aligned[T]` never does; a plain `std::list[T]` is Precog's to place. A narrowed
# list goes anywhere a plain one does, lent or given, and keeps its kind as it grows there; a plain list becomes one
# only by `to.list.page_aligned(…)` / `to.list.not_page_aligned(…)`, which may copy it.
fn total [lent $xs: std::list[i32]] -> i32 {
    let mut $s: i32 = 0;
    for $x in lend $xs { change $s = $s + $x; }
    return $s;
}

fn grow [lent.mut $xs: std::list[i32]] {
    std::list.append(lend.mut $xs, value: 4);
}

MAIN {
    let mut $xs: std::list.page_aligned[i32] = [1, 2, 3];
    grow(lend.mut $xs);
    std::print.useful(total(lend $xs), " ", $xs, newln);
    let $plain: std::list[i32] = [5, 6];
    let $pa := to.list.page_aligned(give $plain);
    let $np: std::list.not_page_aligned[i32] = to.list.not_page_aligned(give $pa);
    let $back: std::list[i32] = give $np;
    std::print.useful($back, newln);
}
