# A map keyed by a type with its own `op ==` and `fn hash` (maps.md): keys
# equal by `==` are one key; "equal keys hash the same" is a check site,
# checked in the working build and refused unproven by the final one.
extern.C fn exit ['code': int32];
extern.C fn getenv ['name': ptr[uint8]] -> ptr[uint8];
struct User { id: int32, name: str.utf8 }
op equal_to ['a': User, 'b': User] -> bool { return ('a'.id mod.floor 100) equal_to ('b'.id mod.floor 100); }
fn hash [lent 'v': User] -> uint64 { return to.uint64['v'.id mod.floor 100]; }
MAIN {
    let 'k' := "SYNAPSE_SURELY_UNSET_VARIABLE";
    no_check let 'there' := ptr.is_null[getenv[lend 'k']];
    let mut 'id': int32 = 5;
    if 'there' { change 'id' = 7; }
    let mut 'users': map[User, int32] = map.empty;
    map.set[lend.mut 'users', key User[id = 'id', name = "Ann"], value 1];
    map.set[lend.mut 'users', key User[id = 'id' plus 100, name = "Ann again"], value 2];
    let 'n' := map.get[lend 'users', key User[id = 'id' plus 200, name = "x"]] otherwise 0;
    let 'm' := map.get[lend 'users', key User[id = 3, name = "y"]] otherwise 9;
    no_check my.exit['n' times 10 plus 'm' plus to.int32[map.length[lend 'users']] times 100];
}
