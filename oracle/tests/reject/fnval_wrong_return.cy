function one [$a: integer.signed.32_bit] -> boolean:
    return true;
end of function one
function apply [$f: function[integer.signed.32_bit] -> integer.signed.32_bit] -> integer.signed.32_bit:
    return $f(1);
end of function apply
!MAIN:
    let $x := apply(one);
!end of MAIN
