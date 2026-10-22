# Empty lists: `[]` with a type, and `list.empty`, then filled with `list.append`.
MAIN {
    let 'xs': std::list[i32] = [];
    std::list.append[lend.mut 'xs', value: 7];
    std::list.append[lend.mut 'xs', value: 8];
    std::print.useful['xs', " ", std::list.length[lend 'xs'], newln];
    let 'ys' := std::list.empty;
    std::print.useful[std::list.length[lend 'ys'], newln];
    std::list.append[lend.mut 'ys', value: "a"];
    std::print.useful['ys', newln];
}
