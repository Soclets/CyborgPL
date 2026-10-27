function look [lent $a: string.utf8] -> integer.signed.32_bit:
    return 1;
end of function look
function apply [$f: function[string.utf8] -> integer.signed.32_bit] -> integer.signed.32_bit:
    return $f("x");
end of function apply
!MAIN:
    let $x := apply(look);
!end of MAIN
