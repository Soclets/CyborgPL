# `while`, `loop` with `break` and `continue`, and `for`.
MAIN {
    let 'i' := 0;
    while 'i' < 3 {
        std::print.useful["while ", 'i', newln];
        change 'i' = 'i' plus 1;
    }
    let 'n' := 0;
    loop {
        change 'n' = 'n' plus 1;
        if 'n' equal_to 2 {
            continue;
        }
        if 'n' > 4 {
            break;
        }
        std::print.useful["loop ", 'n', newln];
    }
    let 'chars' := [`a`, `é`, `🍎`];
    for seq 'c' in lend 'chars' {
        std::print.stored['c', newln];
    }
    std::exit[3];
}
