function one [$a: integer.signed.32_bit] -> integer.signed.32_bit:
    return $a;
end of function one
function apply [$f: function[integer.signed.32_bit, integer.signed.32_bit] -> integer.signed.32_bit] -> integer.signed.32_bit:
    return $f(1, 2);
end of function apply
!MAIN:
    let $x := apply(one);
!end of MAIN
