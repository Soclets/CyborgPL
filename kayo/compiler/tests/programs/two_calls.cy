extern.C fn exit ['code': i32];
struct P { x: i32 }
fn get ['p': P] -> i32 { return 'p'.x; }
MAIN {
    let 'a' := P[x = 5];
    let 'b' := P[x = 9];
    no_check my.exit[get[give 'a'] plus get[give 'b']];
}
