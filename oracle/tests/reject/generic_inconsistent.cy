function max <T: any> [$a: <T>, $b: <T>] -> <T>:
    return $a;
end of function max
!MAIN:
    let $x := max(1, "two");
!end of MAIN
