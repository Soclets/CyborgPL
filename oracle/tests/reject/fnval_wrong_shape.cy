fn one [$a: i32] -> i32 { return $a; }
fn apply [$f: fn[i32, i32] -> i32] -> i32 { return $f(1, 2); }
MAIN {
    let $x := apply(one);
}
