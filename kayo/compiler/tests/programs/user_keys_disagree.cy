# `==` says 7 and 107 are equal, and `hash` gives 7 and 107: the second
# key would sit in another slot. A miss asks every key there is, and the
# comparison that finds them equal stops the program.
%extern.C fn exit [$code: i32];
%extern.C fn getenv [$name: ptr[u8]] -> ptr[u8];
struct User { id: i32, name: str.utf8 }
op equal_to [$a: User, $b: User] -> bool { return ($a.id mod.floor 100) equal_to ($b.id mod.floor 100); }
fn hash [lent $v: User] -> u64 { return to.u64($v.id); }
MAIN {
    let $k := "SYNAPSE_SURELY_UNSET_VARIABLE";
    no_check let $there := ptr.is_null(getenv(lend $k));
    let mut $id: i32 = 5;
    if $there { change $id = 7; }
    let mut $users: std::map[User, i32] = std::map.empty;
    std::map.set(lend.mut $users, key: User[id = $id, name = "Ann"], value: 1);
    std::map.set(lend.mut $users, key: User[id = $id plus 100, name = "Ann again"], value: 2);
    let $n := std::map.get(lend $users, key: User[id = $id plus 200, name = "x"]) otherwise 0;
    let $m := std::map.get(lend $users, key: User[id = 3, name = "y"]) otherwise 9;
    no_check my.exit($n times 10 plus $m plus to.i32(std::map.length(lend $users)) times 100);
}
