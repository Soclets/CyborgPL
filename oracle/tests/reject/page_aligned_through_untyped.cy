# `:=` keeps a list's kind (owner, 2026-10-25: a page-aligned list keeps its kind)
!MAIN:
    let $a: std::ls.page_aligned[int.sign.32] = [5, 6];
    let $b := give $a;
    let $c: std::ls.not_page_aligned[int.sign.32] = give $b;
!end of MAIN
