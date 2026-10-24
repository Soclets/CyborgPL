# nothing calls a `test fn` (owner, 2026-09-29)
test fn t [] { must 1 equal_to 1; }
MAIN { t(); }
