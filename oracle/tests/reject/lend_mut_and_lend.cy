fn two [lent.mut 'a': list[i32], lent 'b': list[i32]] {
    std::print.useful[list.length[lend 'a'] plus list.length[lend 'b'], newln];
}
MAIN {
    let mut 'u': list[i32] = [1, 2];
    two[lend.mut 'u', lend 'u'];
}
