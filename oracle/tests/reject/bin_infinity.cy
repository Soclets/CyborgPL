# a bin never holds an infinity (owner, 2026-09-29: a)
MAIN {
    let $big: f64 = 10.0 pow 308.0;
    let $inf: f64 = $big times 10.0;
}
