# A `mut` name kept changed through `lend.mut`, whatever it holds (check claude, 2026-10-16, corpus 65f59c7a:
# the oracle missed the change when the name held a `"` or a `\`).
fn add [lent.mut 'a': std::list[u16]] { std::list.append[lend.mut 'a', value: 1]; }
MAIN {
    let mut 'o"': std::list[u16] = [];
    add[lend.mut 'o"'];
    let mut 'it\'s': std::list[u16] = [];
    add[lend.mut 'it\'s'];
    let mut 'a\\b': std::list[u16] = [];
    add[lend.mut 'a\\b'];
    std::print.useful[std::list.length[lend 'o"'], " ", std::list.length[lend 'it\'s'], " ", std::list.length[lend 'a\\b'], newln];
}
