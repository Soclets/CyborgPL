function f [$a: integer.signed.64_bit] -> float.%decimal.64_bit:
    let $b := $a times 1000;
    return to.float.%decimal.64_bit($b);
end of function f
!MAIN:
    std::print.shortest(f(5), newline);
!end of MAIN
