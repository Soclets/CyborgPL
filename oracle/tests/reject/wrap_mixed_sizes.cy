# wrapping is at one size, always written: a mixed sum says which (owner, 2026-09-23: b)
MAIN {
    let $a: i8 = 100;
    let $b: i16 = 1000;
    let $w := ($a plus $b) plus.wrap 1;
}
