# Plain math is exact; it is checked where it lands (owner, 2026-09-23)
!MAIN:
    let $v: integer.signed.8_bit = 100;
    let $bad: integer.signed.8_bit = $v plus $v;
!end of MAIN
