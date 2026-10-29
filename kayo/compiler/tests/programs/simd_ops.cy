# simd[T, N] (simd.md): fill, load, store, the lane-wise operators with
# their checks, and the folds all, any, max, sum. Each true case adds its bit.
# A store used to leave Precog believing the list unchanged, and an `if`
# about a stored element was folded to false.
%extern.C proc exit [$code: i32];
fn is_ascii [lent $bytes: arr[u8]] -> bool:
    var $acc: simd[u8, count: 16] = simd.fill(value: 0);
    for $i [from: 1, to: arr.len(lend $bytes), by: 16]:
        change $acc to $acc or.bit simd.load(lend $bytes, pos: $i);
    ;for
    ret simd.max($acc) < 128;
;fn is_ascii
MAIN:
    let $plain: arr[u8] := [72, 101, 108, 108, 111, 32, 119, 111, 114, 108, 100, 33, 33, 33, 33, 33];
    let $thai: arr[u8] := [72, 101, 108, 108, 111, 32, 224, 184, 129, 108, 100, 33, 33, 33, 33, 33];
    var $got: i32 = 0;
    if is_ascii(lend $plain):
        change $got to $got + 1;
    ;if
    if not is_ascii(lend $thai):
        change $got to $got + 2;
    ;if
    let $a: simd[i32, count: 4] := simd.fill(value: 10);
    let $b: simd[i32, count: 4] := simd.fill(value: 3);
    if simd.sum($a) == 40:
        change $got to $got + 4;
    ;if
    if simd.max($a x $b) == 30:
        change $got to $got + 8;
    ;if
    let $m := $a > $b;
    if simd.all($m):
        change $got to $got + 16;
    ;if
    if not simd.any($b > $a):
        change $got to $got + 32;
    ;if
    var $out: arr[i32] = [0, 0, 0, 0, 0, 0];
    simd.store(lend.mut $out, pos: 2, value: $a + $b);
    if $out[pos: 1] == 0 and $out[pos: 2] == 13 and $out[pos: 5] == 13 and $out[pos: 6] == 0:
        change $got to $got + 64;
    ;if
    no_check my.exit($got);
;MAIN
