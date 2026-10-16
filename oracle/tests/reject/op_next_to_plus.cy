op <+> ['a': int32, 'b': int32] -> int32 { return 'a' plus 'b'; }
MAIN {
    let 'w' := 1 plus 2 <+> 3;
}
