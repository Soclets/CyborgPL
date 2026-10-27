function helper [] -> integer.signed.32_bit:
    return 1;
end of function helper

!MAIN:
    let $x := limb.helper[];
!end of MAIN
