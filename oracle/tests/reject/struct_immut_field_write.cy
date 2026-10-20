struct P { immut id: i32, x: i32 }
MAIN {
    let 'p' := P[id = 1, x = 2];
    change 'p'.id = 5;
}
