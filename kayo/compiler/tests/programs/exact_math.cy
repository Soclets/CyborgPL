# Plain math is exact, checked where it lands (owner, 2026-09-23): the
# cases of examples/overflow_sites.cpls, each adding its bit to the exit.
# The last line also pins a wrapping answer kept at its own width when
# stored wider: layer 1 used to fold 8000000000 for 'h'.
%extern.C proc exit [$code: i32];
%extern.C proc strlen [$s: ptr[u8]] -> u64;
MAIN:
    let $z := "ab";
    no_check let $two: i64 := to.i64(strlen(lend $z));
    let $small: i16 := to.i16($two) + 3;
    let $big: i64 := (20000 + 20000) + $small;
    let $s: i16 := 30000;
    let $twice: i64 := $s + $s;
    let $tiny: i8 := (100 + 100) - 150;
    let $v: i8 := 112;
    let $u: u8 := 5;
    let $below := $u - 10;
    let $x: u32 := 4000000000;
    let $h: u64 := $x plus.wrap $x;
    var $got: i64 = 0;
    if $big == 40005:
        change $got to $got + 1;
    ;if
    if $twice == 60000:
        change $got to $got + 2;
    ;if
    if $tiny == 50:
        change $got to $got + 4;
    ;if
    if not ($v > ($v + $v)):
        change $got to $got + 8;
    ;if
    if $below == -5:
        change $got to $got + 16;
    ;if
    if $h == 3705032704:
        change $got to $got + 32;
    ;if
    if $v < 200:
        change $got to $got + 64;
    ;if
    no_check my.exit(to.i32($got));
;MAIN
