# `must.fail` takes something that can fail (owner, 2026-09-29)
test procedure t []:
    let $x := 1; must.fail $x;
end of test procedure t
!MAIN:
!end of MAIN
