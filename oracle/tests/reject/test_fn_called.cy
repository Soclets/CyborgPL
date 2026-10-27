# nothing calls a `test fn` (owner, 2026-09-29)
test proc t []:
    must 1 eq 1;
end of test proc t
!MAIN:
    t();
!end of MAIN
