# A `pub coll.bound` is used in its own kit as any other (design claude, 2026-10-26: as the oracle reads it): exit 3.
pub coll.bound q {
    q = i32[min: 1, max: 9];
}
MAIN {
    let $x: q = 3;
    std::exit(code: $x);
}
