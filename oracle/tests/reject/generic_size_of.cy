function f <T: any> [$a: <T>]:
    let $n := size.of[<T>];
end of function f
!MAIN:
    f(1);
!end of MAIN
