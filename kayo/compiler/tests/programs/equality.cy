# == and !== on an enum (with and without a payload), a struct and a text:
# the same shape all the way down. The instruction used to be dropped
# silently and the answer left unwritten. Each true case adds its bit.
%extern.C fn exit [$code: i32];
%extern.C fn strlen [$s: ptr[u8]] -> u64;
enum Status {
    empty,
    claimed: str.utf8,
    locked
}
struct P { x: i32, y: i32 }
MAIN {
    let $k := "abc";
    no_check let $n: u64 = strlen(lend $k);
    let mut $got: i32 = 0;
    let mut $s := Status.empty;
    if $n > 2 { change $s = Status.locked; }
    if $s equal_to Status.locked { change $got = $got plus 1; }
    if $s not_equal Status.empty { change $got = $got plus 2; }
    let $t := Status.claimed("Bo");
    if $t equal_to Status.claimed("Bo") { change $got = $got plus 4; }
    if $t not_equal Status.claimed("Al") { change $got = $got plus 8; }
    let $a := P[x = to.i32($n), y = 2];
    if $a equal_to P[x = 3, y = 2] { change $got = $got plus 16; }
    if "a" concat "bc" equal_to $k { change $got = $got plus 32; }
    no_check my.exit($got);
}
