fn f <T: any> [$a: <T>] -> <T>:
    ret "x";
end of fn f
!MAIN:
    let $x := f(1);
!end of MAIN
