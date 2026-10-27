function g [$p: integer.unsigned.8_bit] -> integer.unsigned.8_bit:
    if false:
        return true;
    end of if
    return 1;
end of function g
!MAIN:
    let $x := g(1);
!end of MAIN
