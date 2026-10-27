function half_start <T: any> [$x: <T>] -> <T>:
    let $h: <T> = 0.5;
    return $h;
end of function half_start
!MAIN:
    std::print.shortest(half_start(4: integer.signed.32_bit), newline);
!end of MAIN
