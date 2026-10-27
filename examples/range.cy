# Counting loops: `for $i [from: 1, to: 3, by: 1]`, up, down and by steps.
!MAIN:
    for $i [from: 1, to: 3, by: 1]:
        std::print.shortest($i, " ");
    end of for
    std::print.stored(newline);
    for $k [from: 0, to: 10, by: 5]:
        std::print.shortest($k, " ");
    end of for
    std::print.stored(newline);
    for $d [from: 3, to: 1, by: negative 1]:
        std::print.shortest($d, " ");
    end of for
    std::print.stored(newline);
    for permanent $stop [from: 0, to: 100, by: 1]:
        if ($stop times $stop) greater_than 50:
            break;
        end of if
    end of for
    std::print.shortest("stopped at ", $stop, newline);
    for permanent $never [from: 5, to: 2, by: 1]:
        std::print.stored("unreachable", newline);
    end of for
    std::print.shortest("never ran, holds ", $never, newline);
    for permanent $done [from: 0, to: 3, by: 1]:
    end of for
    std::print.shortest("ran out, holds ", $done, newline);
    for $b: integer.signed.8_bit [from: 126, to: 127, by: 1]:
        std::print.shortest($b, " ");
    end of for
    std::print.stored(newline);
    let $words := ["a", "b", "c"];
    for permanent $w in lend $words:
    end of for
    std::print.stored("last word ", $w, newline);
!end of MAIN
