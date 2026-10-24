# unknown_field: a field a letter off.
struct P { width: i32 }
MAIN {
    let $p := P[width = 2];
    std::print.useful($p.widht, newln);
}
