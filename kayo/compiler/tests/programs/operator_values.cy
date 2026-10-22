# examples/operator_values.cpls as an exit code (owner, 2026-09-29): 10 + 24
# + (-10) + (-5) + 2 + 11, and 100 when `op </==` said 3 </== 3 → 132.
extern.C fn exit ['code': i32];
struct Vec { x: f64, y: f64 }
op dot ['a': Vec, 'b': Vec] -> f64 { return 'a'.x times 'b'.x plus 'a'.y times 'b'.y; }
fn fold [lent 'xs': std::list[i32], 'start': i32, 'f': fn[i32, i32] -> i32] -> i32 {
    let mut 'acc' := 'start';
    for 'x' in lend 'xs' { change 'acc' = 'f'['acc', 'x']; }
    return 'acc';
}
fn apply1 ['v': i32, 'f': fn[i32] -> i32] -> i32 { return 'f'[pos: 'v']; }
fn test2 ['a': i32, 'b': i32, 'f': fn[i32, i32] -> bool] -> bool { return 'f'['a', 'b']; }
fn both [lent 'a': Vec, lent 'b': Vec, 'f': fn[lent Vec, lent Vec] -> f64] -> f64 { return 'f'[lend 'a', lend 'b']; }
MAIN {
    let 'xs': std::list[i32] = [1, 2, 3, 4];
    let 'sum' := fold[lend 'xs', 0, op plus];
    let 'product' := fold[lend 'xs', 1, op times];
    let 'down' := fold[lend 'xs', 0, op minus];
    let 'n' := apply1[5, op negate];
    let 'le' := test2[3, 3, op at_most ];
    let 'f': fn[i32, i32] -> i32 = op mod.floor;
    let 'm' := 'f'[-7, 3];
    let 'u' := Vec[x = 1.0, y = 2.0];
    let 'v' := Vec[x = 3.0, y = 4.0];
    let 'd' := both[lend 'u', lend 'v', op dot];
    let 'dn' := to.i32['d'] otherwise 0;
    let mut 'bonus': i32 = 0;
    if 'le' { change 'bonus' = 100; }
    no_check my.exit['sum' plus 'product' plus 'down' plus 'n' plus 'm' plus 'dn' plus 'bonus'];
}
