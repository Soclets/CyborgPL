struct P { immut id: int32, x: int32 }
MAIN {
    let 'p' := P[id = 1, x = 2];
    change 'p'.id = 5;
}
