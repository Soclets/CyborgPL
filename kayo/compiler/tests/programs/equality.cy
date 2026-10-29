# == and !== on an enum (with and without a payload), a struct and a text:
# the same shape all the way down. The instruction used to be dropped
# silently and the answer left unwritten. Each true case adds its bit.
%extern.C proc exit [$code: i32];
%extern.C proc strlen [$s: ptr[u8]] -> u64;
union.tag Status:
    empty;
    claimed: str.utf8;
    locked;
;union.tag Status
rec P:
    var x: i32; var y: i32;
;rec P
MAIN:
    let $k := "abc";
    no_check let $n: u64 := strlen(lend $k);
    var $got: i32 = 0;
    var $s = Status.empty;
    if $n > 2:
        change $s to Status.locked;
    ;if
    if $s == Status.locked:
        change $got to $got + 1;
    ;if
    if $s != Status.empty:
        change $got to $got + 2;
    ;if
    let $t := Status.claimed("Bo");
    if $t == Status.claimed("Bo"):
        change $got to $got + 4;
    ;if
    if $t != Status.claimed("Al"):
        change $got to $got + 8;
    ;if
    let $a := P[x = to.i32($n), y = 2];
    if $a == P[x = 3, y = 2]:
        change $got to $got + 16;
    ;if
    if "a" concat "bc" == $k:
        change $got to $got + 32;
    ;if
    no_check my.exit($got);
;MAIN
