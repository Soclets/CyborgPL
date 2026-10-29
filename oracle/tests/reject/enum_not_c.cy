# only a layout.C enum crosses to C (2026-09-27)
union.tag K:
    a;
    b;
;union.tag K
%extern.C proc f [$k: K];
MAIN:
;MAIN
