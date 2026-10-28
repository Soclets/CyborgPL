# Counting loops: `for $i [from: 1, to: 3, by: 1]`, up, down and by steps.
MAIN:
    for $i [from: 1, to: 3, by: 1]:
        std::print($i, " ");
    ;for
    std::print.stored(newln);
    for $k [from: 0, to: 10, by: 5]:
        std::print($k, " ");
    ;for
    std::print.stored(newln);
    for $d [from: 3, to: 1, by: -1]:
        std::print($d, " ");
    ;for
    std::print.stored(newln);
    for perm $stop [from: 0, to: 100, by: 1]:
        if ($stop x $stop) > 50:
            break;
        ;if
    ;for
    std::print("stopped at ", $stop, newln);
    for perm $never [from: 5, to: 2, by: 1]:
        std::print.stored("unreachable", newln);
    ;for
    std::print("never ran, holds ", $never, newln);
    for perm $done [from: 0, to: 3, by: 1]:
    ;for
    std::print("ran out, holds ", $done, newln);
    for $b: i8 [from: 126, to: 127, by: 1]:
        std::print($b, " ");
    ;for
    std::print.stored(newln);
    let $words := ["a", "b", "c"];
    for perm $w in lend $words:
    ;for
    std::print.stored("last word ", $w, newln);
;MAIN
