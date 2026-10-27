# `<---->` is a spelling of the both-ways binding (owner, 2026-10-06); it checks, then the oracle
# refuses to run assembly.
!MAIN:
    var $v: int.sign.64 := 1;
    !no_check inline.ASM with [$v <----> x0]:
        add x0, x0, #1
    end of inline.ASM
!end of MAIN
