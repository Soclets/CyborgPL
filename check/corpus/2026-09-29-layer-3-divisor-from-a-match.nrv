# Finding 2026-09-29 (check run, seed 1990, shrunk then cut by hand): 'v' is 284 shr 35 = 0, made
# by a match value. Precog claims 5 div 'v' has a nonzero divisor (division_by_zero, layer 3,
# solver), while its own operand range for 'v' is 0 to 536870911. The normal build divides by zero
# and prints 5; the --assert-proofs build stops (proof failed). The oracle stops: division by zero.
extern.C fn write ['fd': int32, 'buf': ptr[uint8], 'len': uint64] -> int64;
fn put_u ['m': uint64] {
    let mut 'buf': list[uint8] = [];
    let mut 'v': uint64 = 'm';
    let mut 'p': uint64 = 1;
    while 'v' div.floor 'p' at_least 10 { change 'p' = 'p' times 10; }
    loop {
        list.append[lend.mut 'buf', value to.uint8[48 plus ('v' div.floor 'p')]];
        change 'v' = 'v' mod.floor 'p';
        if 'p' equal_to 1 { break; }
        change 'p' = 'p' div.floor 10;
    }
    list.append[lend.mut 'buf', value 10];
    no_check write[1, lend 'buf', list.length[lend 'buf']];
}
enum E { a, b: uint64 }
MAIN {
    let 'v': uint64 = match E.b[284] { a { is 1; } b 'd' { is 'd' shr 35; } };
    let 'n': uint64 = 5 div.floor 'v';
    put_u['n'];
}
