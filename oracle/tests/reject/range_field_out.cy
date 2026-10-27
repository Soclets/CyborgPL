rec P:
    var v: int.unsign.8[min: 0, max: 100];
end of rec P
!MAIN:
    let $p := P[v = 120];
!end of MAIN
