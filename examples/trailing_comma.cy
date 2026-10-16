# a trailing `,` is allowed (owner, 2026-10-14)
struct P {
    x: int32,
    y: int32,
}
enum S { a, b, }
MAIN {
    let 'p' := P[x = 1, y = 2];
    std::print.useful['p'.x + 'p'.y, newln];
}
