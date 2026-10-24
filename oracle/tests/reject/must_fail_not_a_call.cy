# `must.fail` takes something that can fail (owner, 2026-09-29)
test fn t [] { let $x := 1; must.fail $x; }
MAIN { }
