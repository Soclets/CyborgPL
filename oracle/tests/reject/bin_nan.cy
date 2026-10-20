# a bin never holds a NaN; 0.0 div.floor 0.0 is a division by zero (owner, 2026-09-29: a)
MAIN {
    let 'z': f64 = 0.0;
    let 'q': f64 = 'z' div.near 'z';
}
