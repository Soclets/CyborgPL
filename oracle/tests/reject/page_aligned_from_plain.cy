# a plain list becomes page-aligned only by a conversion (owner, 2026-10-25)
!MAIN:
    let $plain: std::ls[int.sign.32] = [5, 6];
    let $pa: std::ls.page_aligned[int.sign.32] = give $plain;
!end of MAIN
