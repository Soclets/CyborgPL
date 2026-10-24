module a { struct P { x: i32 } }
MAIN {
    let $p := a.P[x = 1];
}
