# a callback's parameter is a plain number, converted inside (owner, 2026-09-27)
layout.C enum Key { a = 65 }
extern.C fn set_handler [$h: fn[Key]];
MAIN { }
