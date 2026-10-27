# Plain math is exact; it is checked where it lands (owner, 2026-09-23)
!MAIN:
    let $v: int.sign.8 = 100;
    let $bad: int.sign.8 = $v plus $v;
!end of MAIN
