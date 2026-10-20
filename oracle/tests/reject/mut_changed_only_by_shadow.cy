MAIN {
    let mut 'x': i32 = 1;
    shadow 'x': i32 = 2;
    change 'x' = 3;
    std::print.useful['x', newln];
}
