# `while`, `loop` with `break` and `continue`, and `for`.
!MAIN:
    var $i = 0;
    while $i less_than 3:
        std::print.shortest("while ", $i, newln);
        change $i to $i plus 1;
    end of while
    var $n = 0;
    loop:
        change $n to $n plus 1;
        if $n eq 2:
            continue;
        end of if
        if $n greater_than 4:
            break;
        end of if
        std::print.shortest("loop ", $n, newln);
    end of loop
    let $chars := [`a`, `é`, `🍎`];
    for seq $c in lend $chars:
        std::print.stored($c, newln);
    end of for
    std::exit(code: 3);
!end of MAIN
