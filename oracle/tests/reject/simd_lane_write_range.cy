# a lane write outside the vector is a check site, as a list's is (owner, 2026-09-29: a)
MAIN {
    let mut $v: simd[i32, count: 4] = simd.fill(value: 0);
    change $v[pos: 5] = 30;
}
