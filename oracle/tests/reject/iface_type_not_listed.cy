interface Number:
    integer.signed.32_bit;
    float.binary.64_bit;
end of interface Number
function f <T: Number> [$a: <T>] -> <T>:
    return $a;
end of function f
!MAIN:
    let $s := f("text");
!end of MAIN
