# Powers (owner, 2026-10-07): this kit lists `c_calls`, and the program denies every C call except
# `write`, so its one call to `write` builds. No std; CNC for C, and `vouch` for the `no_check`.

%extern.C fn write [$fd: i32, $buf: ptr[u8], $len: u64] -> i64;

MAIN {
    let $text := "allowed
";
    no_check let $n := write(1, lend $text, 8);
}
