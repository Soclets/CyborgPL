# `while`, `loop` with `break` and `continue`, and `for`.
MAIN:
    var $i = 0;
    while $i < 3:
        std::print("while ", $i, newln);
        change $i to $i + 1;
    ;while
    var $n = 0;
    loop:
        change $n to $n + 1;
        if $n == 2:
            continue;
        ;if
        if $n > 4:
            break;
        ;if
        std::print("loop ", $n, newln);
    ;loop
    let $chars := [`a`, `é`, `🍎`];
    for $c in lend $chars:
        std::print.stored($c, newln);
    ;for
    std::exit(code: 3);
;MAIN
