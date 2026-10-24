# A function that calls itself on a larger argument is not proven by
# induction on the arguments: that call is to a tuple not yet done, which
# can be anything the return type holds. up(98) is past an i64 in truth.
extern.C fn exit [$code: i32];
extern.C fn getenv [$name: ptr[u8]] -> ptr[u8];
extern.C fn atoll [$s: ptr[u8]] -> i64[min: 0, max: 100];
fn up [$n: i64[min: 0, max: 100]] -> i64 {
    if $n at_least 100 { return 1; }
    return up($n plus 1) plus 9_000_000_000_000_000_000;
}
fn read_n [$text: ptr[u8]] -> i64[min: 0, max: 100] or fail[missing, out_of_range] {
    if ptr.is_null($text) { fail missing; }
    no_check let $n := atoll($text) otherwise fail;
    return $n;
}
fn run [] -> i32 or fail[missing, out_of_range] {
    let $key := "N";
    no_check let $n := read_n(getenv(lend $key)) otherwise fail;
    return to.i32(up($n) mod.floor 256);
}
MAIN {
    let $code := run() otherwise 2;
    no_check my.exit($code);
}
