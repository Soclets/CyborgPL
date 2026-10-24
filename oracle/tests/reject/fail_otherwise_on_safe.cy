fn f [] -> i32 { return 1; }
MAIN {
    let $x := f() otherwise 0;
}
