# a plain list becomes page-aligned only by a conversion (owner, 2026-10-25)
!MAIN:
    let $plain: std::list[integer.signed.32_bit] = [5, 6];
    let $pa: std::list.page_aligned[integer.signed.32_bit] = give $plain;
!end of MAIN
