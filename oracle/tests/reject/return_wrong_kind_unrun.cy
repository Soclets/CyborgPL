fn g [$p: u8] -> u8 {
    if false { return true; }
    return 1;
}
MAIN {
    let $x := g(1);
}
