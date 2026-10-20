module m {
    fn exit ['x': i32] -> i32 { return 'x'; }
    fn go [] { let 'y' := exit[1]; }
}

MAIN { }
