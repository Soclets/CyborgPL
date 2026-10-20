MAIN {
    let mut 'xs': list[i32] = [1, 2, 3];
    for 'x' in lend.mut 'xs' {
        change 'x' = 'x' plus 1;
        let 'n' := list.length[lend 'xs'];
        std::print.stored['x', newln];
    }
}
