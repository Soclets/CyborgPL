# `shadow`: a new variable with an old name, in an inner block or after.
MAIN {
    let 'x' := 1;
    if true {
        shadow 'x' := 2;
        std::print.useful['x', newln];
    }
    std::print.useful['x', newln];
    shadow 'x': i32 = 3;
    std::print.useful['x', newln];
}
