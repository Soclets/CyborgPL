# deci (owner, 2026-09-29): an i64 could have more digits than a d64 holds, so the conversion says what happens
!MAIN:
    let $big: int.sign.64 := 5;
    let $d := to.float.%deci.64($big);
!end of MAIN
