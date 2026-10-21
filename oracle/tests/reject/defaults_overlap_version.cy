coll.fn say {
    labels ['t' as value, 'k' as count];
    defaults ['k' = 1];
    versions say {
        ['n': i32, 't': i32, 'k': i32] -> i32 { return 1; }
        ['n': i32, 't': i32] -> i32 { return 2; }
    }
}
MAIN { }
