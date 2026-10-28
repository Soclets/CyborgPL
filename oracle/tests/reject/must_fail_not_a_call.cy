# `must.fail` takes something that can fail (owner, 2026-09-29)
test proc t []:
    let $x := 1; must.fail $x;
;test proc t
MAIN:
;MAIN
