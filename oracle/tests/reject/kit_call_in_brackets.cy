function twice [$n: integer.signed.64_bit] -> integer.signed.64_bit:
    return $n times 2;
end of function twice
!MAIN:
    std::print.shortest(twice[2], newline);
!end of MAIN
