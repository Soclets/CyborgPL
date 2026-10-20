op <+> ['a': i32, 'b': i32] -> i32 { return 'a' plus 'b'; }
MAIN {
    let 'w' := 1 plus 2 <+> 3;
}
