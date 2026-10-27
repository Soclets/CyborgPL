# only a layout.C enum crosses to C (2026-09-27)
union.tagged K:
    a; b;
end of union.tagged K
%external.C procedure f [$k: K];
!MAIN:
!end of MAIN
