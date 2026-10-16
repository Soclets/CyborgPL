MAIN {
    let mut 'x': int32 = 1;
    shadow 'x': int32 = 2;
    change 'x' = 3;
    std::print.useful['x', newln];
}
