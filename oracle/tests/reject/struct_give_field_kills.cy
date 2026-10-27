rec P:
    var n: str.utf8;
    var x: int.sign.32;
end of rec P
!MAIN:
    let $p := P[n = "a", x = 2];
    let $t := give $p.n;
    std::print.shortest($p.x);
!end of MAIN
