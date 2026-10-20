fn f [] -> i32 or fail[a] { fail a; }
fn g [] -> i32 or fail[b] { let 'x' := f[] otherwise fail; return 'x'; }
MAIN {
    let 'y' := g[] otherwise 0;
}
