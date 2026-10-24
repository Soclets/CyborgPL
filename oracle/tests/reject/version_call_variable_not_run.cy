coll.fn mx {
    versions mx {
        [$a: i32, $b: i32] -> i32 { return $a + $b; }
        [$a: i32, $b: i64] -> i32 { return $a - to.i32($b); }
    }
}
MAIN {
    let $v: i32 = 4;
    let $p: bool = to.i64(1) equal_to to.i64(2);
    std::print.useful("before", newln);
    if $p {
        std::print.useful(mx($v, 5), newln);
    }
    std::print.useful("after", newln);
}
