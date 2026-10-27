# A list's kinds (owner, 2026-10-25): `std::list.page_aligned[T]` always starts on a page, so a GPU block takes it
# without a copy; `std::list.not_page_aligned[T]` never does; a plain `std::list[T]` is Precog's to place. A narrowed
# list goes anywhere a plain one does, lent or given, and keeps its kind as it grows there; a plain list becomes one
# only by `to.list.page_aligned(…)` / `to.list.not_page_aligned(…)`, which may copy it.
function total [lent $xs: std::list[integer.signed.32_bit]] -> integer.signed.32_bit:
    variable $s: integer.signed.32_bit = 0;
    for $x in lend $xs:
        change $s = $s plus $x;
    end of for
    return $s;
end of function total

procedure grow [lent.mutable $xs: std::list[integer.signed.32_bit]]:
    std::list.append(lend.mutable $xs, value: 4);
end of procedure grow

!MAIN:
    variable $xs: std::list.page_aligned[integer.signed.32_bit] = [1, 2, 3];
    grow(lend.mutable $xs);
    std::print.shortest(total(lend $xs), " ", $xs, newline);
    let $plain: std::list[integer.signed.32_bit] = [5, 6];
    let $pa := to.list.page_aligned(give $plain);
    let $np: std::list.not_page_aligned[integer.signed.32_bit] = to.list.not_page_aligned(give $pa);
    let $back: std::list[integer.signed.32_bit] = give $np;
    std::print.shortest($back, newline);
!end of MAIN
