# A function carrying what it captured with `copy` or `give` is owned like a
# list (owner, 2026-09-24: a): `copy` copies what it carries, `give` hands
# it on, and dropping it gives back what it holds. One carrying nothing, or
# only lends, copies by itself. Lent to a `lent` parameter, it stays put;
# given or copied to a `fn[…]` parameter, the callee owns it and gives it back.
extern.C fn exit ['code': int32];
extern.C fn getenv ['name': ptr[uint8]] -> ptr[uint8];
fn look [lent 'f': fn[] -> uint64] -> uint64 { return 'f'[]; }
fn run ['f': fn[] -> uint64] -> uint64 { return 'f'[]; }
MAIN {
    let 'k' := "SYNAPSE_SURELY_UNSET_VARIABLE";
    no_check let 'there' := ptr.is_null[getenv[lend 'k']];
    let mut 'xs' := [1, 2, 3];
    if 'there' { list.append[lend.mut 'xs', value 4]; }
    let 'count' := fn [] -> uint64 with [copy 'xs'] { return list.length[lend 'xs']; };
    list.append[lend.mut 'xs', value 5];
    let 'again' := copy 'count';
    let 'moved' := give 'count';
    let 'words' := ["a", "bb"];
    let 'size' := fn [] -> uint64 with [give 'words'] { return str.bytes[lend 'words'[pos 2]]; };
    let 'inc' := fn ['a': int32] -> int32 { return 'a' plus 1; };
    let 'inc2' := 'inc';
    let 'inc3' := fn [] -> uint64 { return 7; };
    let mut 'got': int32 = 0;
    if 'again'[] equal_to 4 and 'moved'[] equal_to 4 { change 'got' = 'got' plus 1; }
    if list.length[lend 'xs'] equal_to 5 and 'size'[] equal_to 2 { change 'got' = 'got' plus 2; }
    if 'inc2'[pos 41] equal_to 42 and look[lend 'again'] equal_to 4 { change 'got' = 'got' plus 4; }
    if run[copy 'again'] equal_to 4 and run[give 'again'] equal_to 4 and run['inc3'] equal_to 7 { change 'got' = 'got' plus 8; }
    no_check my.exit['got'];
}
