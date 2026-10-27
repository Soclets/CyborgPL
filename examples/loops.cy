# `while`, `loop` with `break` and `continue`, and `for`.
!MAIN:
    variable $i := 0;
    while $i less_than 3:
        std::print.shortest("while ", $i, newline);
        change $i = $i plus 1;
    end of while
    variable $n := 0;
    loop:
        change $n = $n plus 1;
        if $n equal_to 2:
            continue;
        end of if
        if $n greater_than 4:
            break;
        end of if
        std::print.shortest("loop ", $n, newline);
    end of loop
    let $chars := [`a`, `é`, `🍎`];
    for sequential $c in lend $chars:
        std::print.stored($c, newline);
    end of for
    std::exit(code: 3);
!end of MAIN
