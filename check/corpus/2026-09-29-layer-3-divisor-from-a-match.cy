# Finding 2026-09-29 (check run, seed 1990, shrunk then cut by hand): 'v' is 284 shr 35 = 0, made
# by a match value. Precog claims 5 div 'v' has a nonzero divisor (division_by_zero, layer 3,
# solver), while its own operand range for 'v' is 0 to 536870911. The normal build divides by zero
# and prints 5; the --assert-proofs build stops (proof failed). The oracle stops: division by zero.
%extern.C fn write ['fd': i32, 'buf': ptr[u8], 'len': u64] -> i64;
fn put_u ['m': u64] {
    let mut 'buf': std::list[u8] = [];
    let mut 'v': u64 = 'm';
    let mut 'p': u64 = 1;
    while 'v' div.floor 'p' at_least 10 { change 'p' = 'p' times 10; }
    loop {
        std::list.append(lend.mut 'buf', value: to.u8(48 plus ('v' div.floor 'p')));
        change 'v' = 'v' mod.floor 'p';
        if 'p' equal_to 1 { break; }
        change 'p' = 'p' div.floor 10;
    }
    std::list.append(lend.mut 'buf', value: 10);
    no_check write(1, lend 'buf', std::list.length(lend 'buf'));
}
enum E { a, b: u64 }
MAIN {
    let 'v': u64 = match E.b(284) { a { is 1; } b 'd' { is 'd' shr 35; } };
    let 'n': u64 = 5 div.floor 'v';
    put_u('n');
}
