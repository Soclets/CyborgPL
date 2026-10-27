# Counting loops: `for $i [from: 1, to: 3, by: 1]`, up, down and by steps.
!MAIN:
    for $i [from: 1, to: 3, by: 1]:
        std::print.shortest($i, " ");
    end of for
    std::print.stored(newln);
    for $k [from: 0, to: 10, by: 5]:
        std::print.shortest($k, " ");
    end of for
    std::print.stored(newln);
    for $d [from: 3, to: 1, by: neg 1]:
        std::print.shortest($d, " ");
    end of for
    std::print.stored(newln);
    for perm $stop [from: 0, to: 100, by: 1]:
        if ($stop times $stop) gt 50:
            break;
        end of if
    end of for
    std::print.shortest("stopped at ", $stop, newln);
    for perm $never [from: 5, to: 2, by: 1]:
        std::print.stored("unreachable", newln);
    end of for
    std::print.shortest("never ran, holds ", $never, newln);
    for perm $done [from: 0, to: 3, by: 1]:
    end of for
    std::print.shortest("ran out, holds ", $done, newln);
    for $b: int.sign.8 [from: 126, to: 127, by: 1]:
        std::print.shortest($b, " ");
    end of for
    std::print.stored(newln);
    let $words := ["a", "b", "c"];
    for perm $w in lend $words:
    end of for
    std::print.stored("last word ", $w, newln);
!end of MAIN
