MAIN {
    let 'xs' := [1, 2, 3];          # a list of integers, though nobody wrote the type
    let 'x': f32 = 70.0;
    change 'xs'[pos: 1] = 'x' div.near 2.0;        # a bin into an integer element: a family crossing
}
