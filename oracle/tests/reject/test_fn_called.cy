# nothing calls a `test fn` (owner, 2026-09-29)
test proc t []:
    must 1 == 1;
;test proc t
MAIN:
    t();
;MAIN
