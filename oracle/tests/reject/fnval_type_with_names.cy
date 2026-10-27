fn apply [let $f: fn[$a: int.sign.32] -> int.sign.32] -> int.sign.32:
    ret $f[1];
end of fn apply
!MAIN:
!end of MAIN
