fn f [$a: int.sign.64] -> float.%deci.64:
    let $b := $a times 1000;
    ret to.float.%deci.64($b);
end of fn f
!MAIN:
    std::print.shortest(f(5), newln);
!end of MAIN
