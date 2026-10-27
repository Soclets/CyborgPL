function g [$p: integer.unsigned.8_bit] -> integer.unsigned.8_bit:
    if false:
        shadow $p: boolean = true;
        return $p;
    end of if
    return $p;
end of function g
!MAIN:
    let $x := g(1);
!end of MAIN
