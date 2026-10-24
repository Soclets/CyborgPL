struct P { x: i32 }
fn poke [lent $q: P] {
    change $q.x = 9;
}
MAIN {
    let $p := P[x = 1];
    poke(lend $p);
}
