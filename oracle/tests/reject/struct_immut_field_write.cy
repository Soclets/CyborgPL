rec P:
    let id: int.sign.32;
    var x: int.sign.32;
end of rec P
!MAIN:
    var $p = P[id = 1, x = 2];
    change $p.id to 5;
!end of MAIN
