# Variadics (functions.md): a bundle of any number of values, typed or each
# its own; lent or owned; `...'xs'` unpacking a list (literal, bounded with
# a length known only at run time, or a bundle passed on), `...copy` and
# `...give` of owned elements. The call builds the bundle.
extern.C fn exit ['code': i32];
extern.C fn getenv ['name': ptr[u8]] -> ptr[u8];
fn sum [...'nums'] -> f64 {
    let mut 'total': f64 = 0.0;
    for 'n' in lend 'nums' { change 'total' = 'total' plus to.f64['n']; }
    return 'total';
}
fn add [...'nums': i32] -> i32 {
    let mut 'total': i32 = 0;
    for 'n' in lend 'nums' { change 'total' = 'total' plus 'n'; }
    return 'total';
}
fn count [...'nums': i32] -> u64 { return list.length[lend 'nums']; }
fn bytes [lent 'label': str.utf8, lent ...'parts': str.utf8] -> u64 {
    let mut 'n': u64 = str.bytes[lend 'label'];
    for 'p' in lend 'parts' { change 'n' = 'n' plus str.bytes[lend 'p']; }
    return 'n';
}
fn pass_on [lent ...'parts': str.utf8] -> u64 { return bytes["x", ...'parts']; }
fn keep [...'names': str.utf8] -> u64 { return list.length[lend 'names']; }
MAIN {
    let 'k' := "SYNAPSE_SURELY_UNSET_VARIABLE";
    no_check let 'there' := ptr.is_null[getenv[lend 'k']];
    let mut 'got': i32 = 0;
    let mut 'three': i32 = 4;
    if 'there' { change 'three' = 3; }
    if sum[1, 2.5, 'three'] equal_to 6.5 and sum[] equal_to 0.0 { change 'got' = 'got' plus 1; }
    if add[4, 5, 'three'] equal_to 12 and count[4, 5, 6] equal_to 3 and count[] equal_to 0 { change 'got' = 'got' plus 2; }
    let 'xs' := [1, 2, 'three'];
    if add[...'xs'] equal_to 6 and add[10, ...'xs', 20] equal_to 36 { change 'got' = 'got' plus 4; }
    let mut 'ys': list[i32][min: 0, max: 4] = list.empty;
    list.append[lend.mut 'ys', value: 7];
    if 'there' { list.append[lend.mut 'ys', value: 8]; }
    if add[...'ys'] equal_to 15 and count[...'ys', 1] equal_to 3 { change 'got' = 'got' plus 8; }
    let 'name' := "Ann";
    if bytes["mixed", lend 'name', "yo"] equal_to 10 and bytes["none"] equal_to 4 and pass_on[lend 'name', "b"] equal_to 5 { change 'got' = 'got' plus 16; }
    let mut 'words' := ["a", "bb"];
    if 'there' { change 'words' = ["a", "bcd"]; }
    if keep[...copy 'words'] equal_to 2 and list.length[lend 'words'] equal_to 2 { change 'got' = 'got' plus 32; }
    if keep[...give 'words'] equal_to 2 and list.length[lend 'words'] equal_to 0 { change 'got' = 'got' plus 64; }
    no_check my.exit['got'];
}
