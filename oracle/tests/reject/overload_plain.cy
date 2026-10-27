function f [$a: integer.signed.64_bit] -> integer.signed.64_bit:
    return 1;
end of function f
function f [$b: float.binary.64_bit] -> integer.signed.64_bit:
    return 2;
end of function f
!MAIN:
    std::print.shortest(f(1), newline);
!end of MAIN
