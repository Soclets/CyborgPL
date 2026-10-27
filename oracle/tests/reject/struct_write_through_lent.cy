rec P:
    var x: int.sign.32;
end of rec P
fn poke [lent $q: P]:
    change $q.x to 9;
end of fn poke
!MAIN:
    let $p := P[x = 1];
    poke(lend $p);
!end of MAIN
