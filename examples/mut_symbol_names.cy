# A `mut` name kept changed through `lend.mut`, whatever it holds (check claude, 2026-10-16, corpus 65f59c7a:
# the oracle missed the change when the name held a `"` or a `\`).
fn add [lent.mut 'a': list[u16]] { list.append[lend.mut 'a', value 1]; }
MAIN {
    let mut 'o"': list[u16] = [];
    add[lend.mut 'o"'];
    let mut 'it\'s': list[u16] = [];
    add[lend.mut 'it\'s'];
    let mut 'a\\b': list[u16] = [];
    add[lend.mut 'a\\b'];
    std::print.useful[list.length[lend 'o"'], " ", list.length[lend 'it\'s'], " ", list.length[lend 'a\\b'], newln];
}
