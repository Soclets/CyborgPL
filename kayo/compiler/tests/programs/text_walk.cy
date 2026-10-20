# Walking a text by its units (strings.md): str.each.byte / .point /
# .grapheme — 21 bytes, 8 points, 6 clusters for "aé" ++ "เกี่ยว"; each
# cluster a text of its own, given back with the walk.
extern.C fn exit ['code': i32];
extern.C fn getenv ['name': ptr[u8]] -> ptr[u8];
MAIN {
    let 'k' := "SYNAPSE_SURELY_UNSET_VARIABLE";
    no_check let 'there' := ptr.is_null[getenv[lend 'k']];
    let mut 's' := "ab";
    if 'there' { change 's' = "aé" concat "เกี่ยว"; }
    let mut 'bytes': i32 = 0;
    for 'b' in str.each.byte[lend 's'] { change 'bytes' = 'bytes' plus 1; }
    let mut 'points': i32 = 0;
    let mut 'codes': i64 = 0;
    for 'p' in str.each.point[lend 's'] { change 'points' = 'points' plus 1; change 'codes' = 'codes' plus to.i64[char.code['p']]; }
    let mut 'clusters': i32 = 0;
    let mut 'last' := str.empty;
    for 'g' in str.each.grapheme[lend 's'] { change 'clusters' = 'clusters' plus 1; change 'last' = to.str.utf8['g']; }
    let mut 'got': i32 = 0;
    if 'bytes' equal_to 21 and 'points' equal_to 8 { change 'got' = 'got' plus 1; }
    if 'codes' equal_to 97 plus 233 plus 3648 plus 3585 plus 3637 plus 3656 plus 3618 plus 3623 { change 'got' = 'got' plus 2; }
    if 'clusters' equal_to 6 and 'last' equal_to "ว" { change 'got' = 'got' plus 4; }
    no_check my.exit['got'];
}
