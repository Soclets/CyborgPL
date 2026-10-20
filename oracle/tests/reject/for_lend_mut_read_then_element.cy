MAIN {
    let mut 'xs': list[i32] = [1, 2, 3];
    for 'x' in lend.mut 'xs' {
        let 'first' := 'xs'[pos: 1];
        change 'x' = 'x' plus 'first';
    }
}
