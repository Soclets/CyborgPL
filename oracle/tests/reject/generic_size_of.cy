fn f <T: any> [let $a: <T>]:
    let $n := size.of[<T>];
end of fn f
!MAIN:
    f(1);
!end of MAIN
