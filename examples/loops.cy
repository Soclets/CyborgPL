# `while`, `loop` with `break` and `continue`, and `for`.
MAIN:
    var $i = 0;
    while $i < 3:
        std::println("while ", $i);
        change $i += 1;
    ;while
    var $n = 0;
    loop:
        change $n += 1;
        if $n == 2:
            continue;
        ;if
        if $n > 4:
            break;
        ;if
        std::println("loop ", $n);
    ;loop
    let $chars := [`a`, `é`, `🍎`];
    for seq $c in lend $chars:
        std::println.stored($c);
    ;for
    std::exit(code: 3);
;MAIN
