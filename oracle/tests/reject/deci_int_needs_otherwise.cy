# deci (owner, 2026-09-29): an i64 could have more digits than a d64 holds, so the conversion says what happens
MAIN {
    let $big: i64 = 5;
    let $d := to.d64($big);
}
