# Finding 2026-10-16 (check run, seed 351, cut down by hand): a list lent mutably to a function that
# changes it keeps its `mut`, but the oracle says "'o"' is `mut`, but nothing changes it" when the name
# holds a `"` or a `\` (written `\\`). With 'o', 'o%', 'o.x', 'o a' or 'o\'' it prints 1, as kayo does
# for all of them (owner, 2026-10-16: a quoted name holds any printable character, c61b99ac).
fn add [lent.mut 'a': list[u16]] {
    list.append[lend.mut 'a', value: 1];
}
MAIN {
    let mut 'o"': list[u16] = [];
    add[lend.mut 'o"'];
    std::print.useful[list.length[lend 'o"'], newln];
    let mut 'b\\': list[u16] = [];
    add[lend.mut 'b\\'];
    std::print.useful[list.length[lend 'b\\'], newln];
}
