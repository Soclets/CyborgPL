fn sum [...$n: int.sign.32] -> int.sign.32:
    ret 0;
end of fn sum
!MAIN:
    let $x := [1, 2]; let $t := sum(...copy $x);
!end of MAIN
