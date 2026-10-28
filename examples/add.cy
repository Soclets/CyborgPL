# A function that adds two numbers, called from MAIN.
fn add [let $a: i32, let $b: i32] -> i32:
    ret $a + $b;
;fn add

MAIN:
    let $sum := add(10, 20);
    std::print("sum: ", $sum, newln);
;MAIN
