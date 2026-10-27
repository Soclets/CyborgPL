function f <T: Nope> [$a: <T>] -> <T>:
    return $a;
end of function f
!MAIN:
    let $x := f(1);
!end of MAIN
