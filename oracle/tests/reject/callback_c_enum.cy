# a callback's parameter is a plain number, converted inside (owner, 2026-09-27)
%layout.C enum Key:
    a = 65;
;%layout.C enum Key
%extern.C proc set_handler [$h: fn[Key]];
MAIN:
;MAIN
