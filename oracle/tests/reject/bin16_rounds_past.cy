# a value that rounds past f16's largest (65504) does not fit (2026-09-24)
MAIN {
    let $x: f16 = 65520.0;
}
