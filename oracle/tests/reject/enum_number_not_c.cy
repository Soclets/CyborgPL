# only a layout.C enum has numbers (2026-09-28)
union.tag K:
    a;
end of union.tag K
!MAIN:
    let $n := to.int.sign.32(K.a);
!end of MAIN
