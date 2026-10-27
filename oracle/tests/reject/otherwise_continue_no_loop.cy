fn small [let $n: int.sign.32] -> int.sign.32 or fail[bad]:
    if $n gt 2:
        fail bad;
    end of if
    ret $n;
end of fn small

!MAIN:
    let $x := small(1) otherwise continue;
    std::print.shortest($x, newln);
!end of MAIN
