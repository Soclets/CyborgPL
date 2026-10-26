fn f [$a: i64] -> %d64 {
    let $b := $a times 1000;
    return to.%d64($b);
}
MAIN {
    std::print.useful(f(5), newln);
}
