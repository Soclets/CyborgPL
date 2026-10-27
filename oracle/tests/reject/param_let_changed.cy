fn countdown [let $n: int.sign.32] -> int.sign.32:
    change $n = $n minus 1;
    ret $n;
end of fn countdown
!MAIN:
!end of MAIN
