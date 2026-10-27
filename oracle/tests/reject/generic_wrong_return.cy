function f <T: any> [$a: <T>] -> <T>:
    return "x";
end of function f
!MAIN:
    let $x := f(1);
!end of MAIN
