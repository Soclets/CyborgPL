rec Point:
    var x: int.sign.32;
    var y: int.sign.32;
end of rec Point
!MAIN:
    let $p := Point[x = 3, y = 4];
    let $t := to.str.utf8($p);
!end of MAIN
