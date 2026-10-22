fn two [lent.mut 'a': std::list[i32], lent 'b': std::list[i32]] {
    std::print.useful[std::list.length[lend 'a'] plus std::list.length[lend 'b'], newln];
}
MAIN {
    let mut 'u': std::list[i32] = [1, 2];
    two[lend.mut 'u', lend 'u'];
}
