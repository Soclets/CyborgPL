rec User:
    var name: str.utf8;
;rec User
op == [lent $a: User, lent $b: User] -> bool:
    ret std::str.lower.full(lend $a.name) == std::str.lower.full(lend $b.name);
;op ==
fn hash [lent $v: User] -> u64:
    let $first := std::str.at.point(lend $v.name, pos: 1) ?? `?`;
    ret char.code($first);          # case-sensitive: "Ann" and "ann" hash differently, but equal_to says equal
;fn hash
MAIN:
    var $users: std::map[User, i32] = std::map.empty;
    std::map.set(lend.mut $users, key: User[name = "Ann"], value: 1);
    let $n := std::map.get(lend $users, key: User[name = "ann"]) ?? 0;
;MAIN
