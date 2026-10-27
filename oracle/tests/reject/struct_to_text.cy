rec Point:
    x: int.sign.32;
    y: int.sign.32;
end of rec Point
!MAIN:
    let $p := Point[x = 3, y = 4];
    let $t := to.str.utf8($p);
!end of MAIN
