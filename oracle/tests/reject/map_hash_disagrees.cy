struct User { name: str.utf8 }
op equal_to ['a': User, 'b': User] -> bool { return std::str.lower.full[lend 'a'.name] equal_to std::str.lower.full[lend 'b'.name]; }
fn hash [lent 'v': User] -> u64 {
    let 'first' := std::str.at.point[lend 'v'.name, pos: 1] otherwise `?`;
    return char.code['first'];          # case-sensitive: "Ann" and "ann" hash differently, but equal_to says equal
}
MAIN {
    let mut 'users': map[User, i32] = map.empty;
    map.set[lend.mut 'users', key: User[name = "Ann"], value: 1];
    let 'n' := map.get[lend 'users', key: User[name = "ann"]] otherwise 0;
}
