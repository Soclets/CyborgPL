# Functions as values (functions-as-values.md): a named function by its
# bare name, held in a variable and called as 'f'[…], passed to a function
# that takes a `fn[…]`; a literal capturing nothing, and one borrowing with
# `with [lend …]`, called from inside a loop. Values chosen at run time.
extern.C fn exit ['code': int32];
extern.C fn getenv ['name': ptr[uint8]] -> ptr[uint8];
struct Locker { number: int32, owner: str.utf8 }
fn by_number [lent 'a': Locker, lent 'b': Locker] -> bool { return 'a'.number < 'b'.number; }
fn by_owner [lent 'a': Locker, lent 'b': Locker] -> bool { return str.bytes[lend 'a'.owner] < str.bytes[lend 'b'.owner]; }
fn first_by [lent 'xs': list[Locker], 'less': fn[lent Locker, lent Locker] -> bool] -> int32 {
    let mut 'best' := 1;
    for 'i' from 2 to list.length[lend 'xs'] by 1 {
        if 'less'[lend 'xs'[pos 'i'], lend 'xs'[pos 'best']] { change 'best' = 'i'; }
    }
    let 'winner' := lend 'xs'[pos 'best'];
    return 'winner'.number;
}
fn count_if [lent 'xs': list[int32], 'test': fn[int32] -> bool] -> int32 {
    let mut 'n': int32 = 0;
    for 'x' in lend 'xs' { if 'test'[pos 'x'] { change 'n' = 'n' plus 1; } }
    return 'n';
}
MAIN {
    let 'k' := "SYNAPSE_SURELY_UNSET_VARIABLE";
    no_check let 'there' := ptr.is_null[getenv[lend 'k']];
    let mut 'got': int32 = 0;
    let mut 'first': int32 = 9;
    if 'there' { change 'first' = 7; }
    let 'lockers' := [Locker[number = 'first', owner = "Cy"], Locker[number = 2, owner = "Annabel"], Locker[number = 5, owner = "Bo"]];
    if first_by[lend 'lockers', by_number] equal_to 2 { change 'got' = 'got' plus 1; }
    if first_by[lend 'lockers', by_owner] equal_to 'first' { change 'got' = 'got' plus 2; }
    let 'f' := by_number;
    if 'f'[lend 'lockers'[pos 2], lend 'lockers'[pos 1]] { change 'got' = 'got' plus 4; }
    let mut 'limit' := 60;
    if 'there' { change 'limit' = 50; }
    let 'scores' := [72, 40, 95, 55];
    if count_if[lend 'scores', fn ['s': int32] -> bool with [lend 'limit'] { return 's' at_least 'limit'; }] equal_to 3 { change 'got' = 'got' plus 8; }
    if count_if[lend 'scores', fn ['s': int32] -> bool { return ('s' mod.floor 2) equal_to 0; }] equal_to 2 { change 'got' = 'got' plus 16; }
    let 'g' := fn ['s': int32] -> bool with [lend 'limit'] { return 's' < 'limit'; };
    let mut 'h' := 'g';
    if 'there' { change 'h' = fn ['s': int32] -> bool { return 's' > 90; }; }
    if count_if[lend 'scores', 'h'] equal_to 1 and count_if[lend 'scores', 'g'] equal_to 1 { change 'got' = 'got' plus 32; }
    no_check my.exit['got'];
}
