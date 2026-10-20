bare.never [fn];

module text {
    extern.C {
        pub fn strlen ['s': ptr[u8]] -> u64;
        fn write ['fd': i32, 'buf': ptr[u8], 'len': u64] -> i64;
    }
}
MAIN {
    let 'msg' := "hi";
    no_check let 'n' := text.write[1, lend 'msg', 2];
}
