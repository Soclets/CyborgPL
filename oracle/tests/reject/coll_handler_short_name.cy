fn half [$n: i32] -> i32 or fail[odd] {
    if $n x 1 equal_to 3 { fail odd; }
    return $n;
}

coll.handler h {
    as_zero {
        odd { return 0; }
    }
}

MAIN { }
