# fib on 0 to 93, N from outside: fib(93) does not fit an i64, so the
# final build must refuse where it lands.
%extern.C fn exit [$code: i32];
%extern.C fn getenv [$name: ptr[u8]] -> ptr[u8];
%extern.C fn atoll [$s: ptr[u8]] -> i64[min: 0, max: 93];
fn fib [$n: i64[min: 0, max: 93]] -> i64 {
    if $n < 2 { return $n; }
    return fib($n minus 1) plus fib($n minus 2);
}
fn read_n [$text: ptr[u8]] -> i64[min: 0, max: 93] or fail[missing, out_of_range] {
    if ptr.is_null($text) { fail missing; }
    no_check let $n := atoll($text) otherwise fail;
    return $n;
}
fn run [] -> i32 or fail[missing, out_of_range] {
    let $key := "N";
    no_check let $n := read_n(getenv(lend $key)) otherwise fail;
    return to.i32(fib($n) mod.floor 256);
}
MAIN {
    let $code := run() otherwise 2;
    no_check my.exit($code);
}
