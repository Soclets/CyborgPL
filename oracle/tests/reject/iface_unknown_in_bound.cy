fn f <T: Nope> [$a: <T>] -> <T>:
    ret $a;
end of fn f
!MAIN:
    let $x := f(1);
!end of MAIN
