rec P:
    let id: int.sign.32;
    x: int.sign.32;
end of rec P
!MAIN:
    var $p := P[id = 1, x = 2];
    change $p.id = 5;
!end of MAIN
