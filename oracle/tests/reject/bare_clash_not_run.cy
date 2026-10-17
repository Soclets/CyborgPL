module m {
    fn exit ['x': int32] -> int32 { return 'x'; }
    fn go [] { let 'y' := exit[1]; }
}

MAIN { }
