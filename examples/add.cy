# A function that adds two numbers, called from MAIN.
function add [$a: integer.signed.32_bit, $b: integer.signed.32_bit] -> integer.signed.32_bit:
    return $a plus $b;
end of function add

!MAIN:
    let $sum := add(10, 20);
    std::print.shortest("sum: ", $sum, newline);
!end of MAIN
