record User:
    name: string.utf8;
end of record User
operator equal_to [$a: User, $b: User] -> boolean:
    return std::string.lowercase.full(lend $a.name) equal_to std::string.lowercase.full(lend $b.name);
end of operator equal_to
function hash [lent $v: User] -> integer.unsigned.64_bit:
    let $first := std::string.at.code_point(lend $v.name, position: 1) otherwise `?`;
    return character.code($first);          # case-sensitive: "Ann" and "ann" hash differently, but equal_to says equal
end of function hash
!MAIN:
    variable $users: std::map[User, integer.signed.32_bit] = std::map.empty;
    std::map.set(lend.mutable $users, key: User[name = "Ann"], value: 1);
    let $n := std::map.get(lend $users, key: User[name = "ann"]) otherwise 0;
!end of MAIN
