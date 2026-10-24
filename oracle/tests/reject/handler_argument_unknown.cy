fn half [$n: i32] -> i32 or fail[odd, negative: i32] {
    if $n < 0 { fail negative[$n]; }
    if ($n div.floor 2) x 2 not_equal $n { fail odd; }
    return $n div.floor 2;
}

coll.handler half_failed {
    half_failed.or_default [$default: i32, $label: str.utf8] {
        odd { std::print.useful($label, ": odd, using ", $default, newln); return $default; }
        negative $v { std::print.useful($label, ": ", $v, " is negative, using ", $default, newln); return $default; }
    }
}

fn half_or [$n: i32, $d: i32] -> i32 {
    let $h := half($n) otherwise half_failed.or_default(default: $d, label: "x", code: 3);
    return $h;
}

MAIN {
    std::print.useful(half_or(8, 0), " ", half_or(7, 100), " ", half_or(-4, 5 x 2), newln);
}
