# The upper half of a u64 is a big number, not a negative one: compare,
# divide and widen it as unsigned. From C, so it is decided at run time.
extern.C fn exit [$code: i32];
extern.C fn labs [$v: i64] -> i64;
MAIN {
    no_check let $one: i64 = labs(-1);
    no_check let $u: u64 = to.u64($one);
    let $big: u64 = $u times 10000000000000000000;     # 10^19: above 2^63
    let mut $score: i32 = 0;
    if $big > $u { change $score = $score plus 1; }             # unsigned compare
    let $half: u64 = $big div.floor 2;                      # unsigned divide
    if $half equal_to 5000000000000000000 { change $score = $score plus 10; }
    let $small: u8 = 250;
    let mut $n: i32 = 0;
    for $i: u8 [from: $small, to: 255, by: 3] { change $n = $n plus 1; }   # 250, 253: widening a u8 is not sign-extending it
    no_check my.exit($score plus $n times 100);
}
