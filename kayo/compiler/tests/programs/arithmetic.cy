# CCA's own test programs, not the language's examples (those are the design
# session's). Each one exits with a number this test knows.
%extern.C proc exit [$code: i32];

fn add [$a: i32, $b: i32] -> i32:
    ret $a + $b;
;fn add
fn twice [$n: i32] -> i32:
    ret $n x 2;
;fn twice

MAIN:
    let $sum: i32 := add(20, 1);
    no_check my.exit(twice($sum));       # 42
;MAIN
