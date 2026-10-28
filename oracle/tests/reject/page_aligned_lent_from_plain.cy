# a lend into a page-aligned parameter must be of that kind (owner, 2026-10-25)
proc f [lent.mut $xs: std::ls.page_aligned[i32]]:
;proc f
MAIN:
    var $plain: std::ls[i32] = [5, 6];
    f(lend.mut $plain);
;MAIN
