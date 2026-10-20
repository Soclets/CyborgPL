fn grow [@y] [lent.mut @y 'xs': list[i32]] -> lent.mut @y list[i32] { list.append[lend.mut 'xs', value 9]; return lend.mut 'xs'; }
fn two [lent.mut 'a': list[i32], lent 'b': list[i32]] {
    std::print.useful[list.length[lend 'a'] plus list.length[lend 'b'], newln];
}
MAIN {
    let mut 'u': list[i32] = [1, 2];
    two[grow[lend.mut 'u'], lend 'u'];
}
