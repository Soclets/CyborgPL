# only a layout.C enum has numbers (2026-09-28)
union.tagged K:
    a;
end of union.tagged K
!MAIN:
    let $n := to.integer.signed.32_bit(K.a);
!end of MAIN
