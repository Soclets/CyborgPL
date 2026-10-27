rec User:
    var name: str.utf8;
end of rec User
op eq [$a: User, $b: User] -> bool:
    ret std::str.lower.full(lend $a.name) eq std::str.lower.full(lend $b.name);
end of op eq
fn hash [lent $v: User] -> int.unsign.64:
    let $first := std::str.at.point(lend $v.name, pos: 1) otherwise `?`;
    ret char.code($first);          # case-sensitive: "Ann" and "ann" hash differently, but equal_to says equal
end of fn hash
!MAIN:
    var $users: std::map[User, int.sign.32] = std::map.empty;
    std::map.set(lend.mut $users, key: User[name = "Ann"], value: 1);
    let $n := std::map.get(lend $users, key: User[name = "ann"]) otherwise 0;
!end of MAIN
