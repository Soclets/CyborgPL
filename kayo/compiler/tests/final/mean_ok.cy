# An average of i32 values fits an i32 (precog.md: "a sum bounded by n x
# max divided by n is at most max"), for a list whose length and contents
# come from outside. The final build accepts it with no checks left.
extern.C fn exit ['code': i32];
extern.C fn getenv ['name': ptr[u8]] -> ptr[u8];
extern.C fn atoi ['s': ptr[u8]] -> i32;
fn average [lent 'xs': list[i32]] -> i32 or fail[empty] {
    if list.length[lend 'xs'] equal_to 0 { fail empty; }
    let mut 'total' := 0;
    for 'x' in lend 'xs' { change 'total' = 'total' plus 'x'; }
    return 'total' div.floor list.length[lend 'xs'];
}
MAIN {
    let 'k' := "N";
    no_check let 'n': i32 = atoi[getenv[lend 'k']];
    let mut 'xs': list[i32][min 0, max 1000] = list.empty;
    let mut 'i': i32 = 0;
    while 'i' < 'n' and 'i' < 1000 {
        list.append[lend.mut 'xs', value ('n' mod.floor 1000) minus 'i'];
        change 'i' = 'i' plus 1;
    }
    let 'a' := average[lend 'xs'] otherwise 0;
    no_check my.exit['a' mod.floor 256];
}
