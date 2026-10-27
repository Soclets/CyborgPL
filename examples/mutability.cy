# Plain, `mut` and `immut` variables, and an `immut` parameter.
function twice [$n: integer.signed.32_bit] -> integer.signed.32_bit:
    return $n times 2;
end of function twice

!MAIN:
    variable $plain := 1;
    change $plain = $plain plus 1;
    variable $counter := 0;
    change $counter = $counter plus 1;
    let $limit: integer.signed.32_bit = 10;
    std::print.shortest($plain, " ", $counter, " ", $limit, " ", twice($limit), newline);
!end of MAIN
