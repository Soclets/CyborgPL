# a lend into a page-aligned parameter must be of that kind (owner, 2026-10-25)
procedure f [lent.mutable $xs: std::list.page_aligned[integer.signed.32_bit]]:
end of procedure f
!MAIN:
    variable $plain: std::list[integer.signed.32_bit] = [5, 6];
    f(lend.mutable $plain);
!end of MAIN
