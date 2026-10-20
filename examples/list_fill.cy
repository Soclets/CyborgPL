# A list of n copies of one value (owner, 2026-09-30): `list.fill[value value, count count]`. A strict file
# labels the count, `list.fill[value 0, count 10]`, as a position says `pos`.
MAIN {
    let mut 'scores' := list.fill[value 0, count 5];
    change 'scores'[pos 2] = 7;
    std::print.stored['scores', newln];
    let 'n': i64 = 3;
    let 'flags': list[bool] = list.fill[value false, count 'n'];
    std::print.stored['flags', " ", list.length[lend 'flags'], newln];
    let 'none': list[i32] = list.fill[value 1, count 0];
    std::print.stored[list.length[lend 'none'], newln];
}
