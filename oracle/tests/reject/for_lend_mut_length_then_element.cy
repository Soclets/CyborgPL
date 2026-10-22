MAIN {
    let mut 'xs': std::list[i32] = [1, 2, 3];
    for 'x' in lend.mut 'xs' {
        change 'x' = 'x' plus 1;
        let 'n' := std::list.length[lend 'xs'];
        std::print.stored['x', newln];
    }
}
