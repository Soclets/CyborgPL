# nothing calls a `test fn` (owner, 2026-09-29)
test procedure t []:
    must 1 equal_to 1;
end of test procedure t
!MAIN:
    t();
!end of MAIN
