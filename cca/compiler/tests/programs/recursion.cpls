extern.C fn exit ['code': int32];

fn fib ['n': int32] -> int32 {
    if 'n' < 2 { return 'n'; }
    return fib['n' minus 1] plus fib['n' minus 2];
}

MAIN {
    no_check my.exit[fib[10]];                            # 55
}
