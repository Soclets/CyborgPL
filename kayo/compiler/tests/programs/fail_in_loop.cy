# A failing call in a loop, twenty million times. Its outcome had a stack
# slot made where the call was, which LLVM reads as a new allocation each
# trip: the stack grew until the program died (exit 139). Every slot is
# now made once, in the function's first block. The sum of 1 to 10^7 is
# 50000005000000; 0 mod 1000, and 7 is the exit.
extern.C fn exit [$code: i32];
fn half [$n: i64] -> i64 or fail[odd] {
    if ($n mod.floor 2) equal_to 1 { fail odd; }
    return $n div.floor 2;
}
MAIN {
    let mut $acc: i64 = 0;
    for $i: i64 [from: 1, to: 20000000, by: 1] {
        let $h := half($i) otherwise 0;
        change $acc = ($acc plus $h) mod.floor 1000;
    }
    no_check my.exit(to.i32($acc mod.floor 100) plus 7);
}
