struct P { n: str.utf8, x: i32 }
MAIN {
    let $p := P[n = "a", x = 2];
    let $t := give $p.n;
    std::print.useful($p.x);
}
