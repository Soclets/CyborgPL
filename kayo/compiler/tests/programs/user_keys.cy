# A map keyed by a type with its own `op ==` and `fn hash` (maps.md): keys
# equal by `==` are one key; "equal keys hash the same" is a check site,
# checked in the working build and refused unproven by the final one.
%extern.C proc exit [$code: i32];
%extern.C proc getenv [$name: ptr[u8]] -> ptr[u8];
rec User:
    var id: i32; var name: str.utf8;
;rec User
op == [lent $a: User, lent $b: User] -> bool:
    ret ($a.id mod.floor 100) == ($b.id mod.floor 100);
;op ==
fn hash [lent $v: User] -> u64:
    ret to.u64($v.id mod.floor 100);
;fn hash
MAIN:
    let $k := "SYNAPSE_SURELY_UNSET_VARIABLE";
    no_check let $there := ptr.is_null(getenv(lend $k));
    var $id: i32 = 5;
    if $there:
        change $id to 7;
    ;if
    var $users: std::map[User, i32] = std::map.empty;
    std::map.set(lend.mut $users, key: User[id = $id, name = "Ann"], value: 1);
    std::map.set(lend.mut $users, key: User[id = $id + 100, name = "Ann again"], value: 2);
    let $n := std::map.get(lend $users, key: User[id = $id + 200, name = "x"]) ?? 0;
    let $m := std::map.get(lend $users, key: User[id = 3, name = "y"]) ?? 9;
    no_check my.exit($n x 10 + $m + to.i32(std::map.count(lend $users)) x 100);
;MAIN
