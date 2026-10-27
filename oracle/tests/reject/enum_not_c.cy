# only a layout.C enum crosses to C (2026-09-27)
union.tag K:
    a;
    b;
end of union.tag K
%extern.C proc f [let $k: K];
!MAIN:
!end of MAIN
