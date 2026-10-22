fn grow [@y] [lent.mut @y 'xs': std::list[i32]] -> lent.mut @y std::list[i32] { std::list.append[lend.mut 'xs', value: 9]; return lend.mut 'xs'; }
fn two [lent.mut 'a': std::list[i32], lent 'b': std::list[i32]] {
    std::print.useful[std::list.length[lend 'a'] plus std::list.length[lend 'b'], newln];
}
MAIN {
    let mut 'u': std::list[i32] = [1, 2];
    two[grow[lend.mut 'u'], lend 'u'];
}
