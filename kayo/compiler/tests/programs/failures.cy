extern.C fn exit ['code': i32];
fn half ['n': i32] -> i32 or fail[odd, negative: i32] {
    if 'n' < 0 { fail negative['n']; }
    if ('n' mod.floor 2) equal_to 1 { fail odd; }
    return 'n' div.floor 2;
}
MAIN {
    let 'a' := half[10] otherwise 0;
    let 'b' := half[7] otherwise 100;
    let mut 'c': i32 = 0;
    half[-4] otherwise {
        odd          { change 'c' = 1; }
        negative 'v' { change 'c' = 0 minus 'v'; }
    };
    no_check my.exit['a' plus 'b' plus 'c'];
}
