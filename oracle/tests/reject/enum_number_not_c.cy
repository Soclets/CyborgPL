# only a layout.C enum has numbers (2026-09-28)
union.tag K:
    a;
;union.tag K
MAIN:
    let $n := to.i32(K.a);
;MAIN
