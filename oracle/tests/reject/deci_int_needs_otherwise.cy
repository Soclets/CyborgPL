# deci (owner, 2026-09-29): an i64 could have more digits than a d64 holds, so the conversion says what happens
!MAIN:
    let $big: integer.signed.64_bit = 5;
    let $d := to.float.%decimal.64_bit($big);
!end of MAIN
