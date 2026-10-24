fn big [$a: i32] -> i32 {
    return $a times $a;
}
MAIN {
    let $r := big(100000);
}
