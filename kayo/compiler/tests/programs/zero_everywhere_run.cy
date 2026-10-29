# No bin or brain16 result is ever -0 (owner, 2026-10-07: "a"); each `.ieee`
# form, and `neg.ieee`, keeps IEEE's ("yea"). Each zero's text, "0" or "-0",
# tells its sign: `plus` (+0) adds 1 and `ieee` (-0) adds 2 when
# right, 0 when wrong. The oracle gives 56. `mod` and `pow` too
# (owner, 2026-10-07: "yea.").
# Values come from calls so nothing is folded, then once more as literals.
%extern.C proc exit [let $code: i32];
fn zero [] -> f64:
    ret 0.0;
;fn zero
fn two [] -> f64:
    ret -2.0;
;fn two
fn three [] -> f64:
    ret 3.0;
;fn three
fn tiny [] -> f64:
    ret 0.000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000001;
;fn tiny
fn sign [let $q: f64] -> i32:
    let $t := to.shortest.str.utf8($q);
    if std::str.bytes(lend $t) == 1:
        ret 1;
    ;if
    ret 0;
;fn sign
fn + [let $q: f64] -> i32:
    ret sign($q);
;fn +
fn ieee [let $q: f64] -> i32:
    ret 2 x (1 - sign($q));
;fn ieee
MAIN:
    let $z := zero();
    let $n := two();
    let $t := tiny();
    let $m: f64 := neg.ieee $z;
    let $a := +($z x $n) + ieee($z times.ieee $n);
    let $b := +(-$z) + ieee(neg.ieee $z);
    let $c := +($t x (-$t)) + ieee($t times.ieee (-$t));
    let $d := +($z div.near $n) + ieee($z div.near.ieee $n) + +($z div.ceil $n) + ieee($z div.ceil.ieee $n);
    let $e := +($m + $m) + ieee($m plus.ieee $m) + +($m - $z) + ieee($m minus.ieee $z);
    let $f := +(3.0 mod.floor (-3.0)) + +($n mod.floor $n) + ieee($m);
    let $bz: %bf16 := 0.0;
    let $bc: %bf16 := -3.0;
    let $g := +(to.f64($bz x $bc)) + ieee(to.f64($bz times.ieee $bc)) + +(to.f64(-$bz)) + ieee(to.f64(neg.ieee $bz));
    let $w: f32 := to.f32($t x (-1.0));
    let $k: f32 := to.f32($m);
    let $h := +(to.f64($w)) + ieee(to.f64($k)) + +(to.f64(to.%bf16($t x (-1.0))));
    let $vz: simd[f64, count: 2] := simd.fill(value: $z);
    let $vm: simd[f64, count: 2] := simd.fill(value: $m);
    let $vn: simd[f64, count: 2] := simd.fill(value: $n);
    let $s := +(simd.max($vm)) + +(simd.min($vm)) + +(simd.sum($vm)) + +(simd.max($vz x $vn)) + +(simd.max($vz times.ieee $vn));
    let $r := three();
    let $x := +((-$r) mod.trunc $r) + ieee((-$r) mod.trunc.ieee $r) + ieee($r mod.floor.ieee (-$r)) + +((-$t) std::pow 3) + ieee((-$t) std::pow.ieee 3) + ieee((-$t) std::pow.ieee 3);
    let $lit := +(0.0 x (-2.0)) + ieee(0.0 times.ieee (-2.0)) + +(-0.0) + ieee(neg.ieee 0.0);
    no_check my.exit(to.i32($a + $b + $c + $d + $e + $f + $g + $h + $s + $x + $lit));
;MAIN
