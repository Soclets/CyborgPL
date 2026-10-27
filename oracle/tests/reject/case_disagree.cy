case.any [function];
case.exact function Area [] -> integer.signed.32_bit:
    return 1;
end of case
!MAIN:
    let $a := Area();
!end of MAIN
