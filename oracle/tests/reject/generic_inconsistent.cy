fn max <T: any> [$a: <T>, $b: <T>] -> <T>:
    ret $a;
end of fn max
!MAIN:
    let $x := max(1, "two");
!end of MAIN
