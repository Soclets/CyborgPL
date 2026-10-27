# a `test fn` takes no parameters (owner, 2026-09-29)
test proc t [$x: int.sign.32]:
    must $x eq 1;
end of test proc t
!MAIN:
!end of MAIN
