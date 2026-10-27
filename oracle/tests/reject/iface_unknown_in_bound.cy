fn f <T: Nope> [let $a: <T>] -> <T>:
    ret $a;
end of fn f
!MAIN:
    let $x := f(1);
!end of MAIN
