# Counting loops: `for $i [from: 1, to: 3, by: 1]`, up, down and by steps.
MAIN:
    for $i [from: 1, to: 3, by: 1]:
        std::print($i, " ");
    ;for
    std::println.stored();
    for $k [from: 0, to: 10, by: 5]:
        std::print($k, " ");
    ;for
    std::println.stored();
    for $d [from: 3, to: 1, by: -1]:
        std::print($d, " ");
    ;for
    std::println.stored();
    for perm $stop [from: 0, to: 100, by: 1]:
        if ($stop x $stop) > 50:
            break;
        ;if
    ;for
    std::println("stopped at ", $stop);
    for perm $never [from: 5, to: 2, by: 1]:
        std::println.stored("unreachable");
    ;for
    std::println("never ran, holds ", $never);
    for perm $done [from: 0, to: 3, by: 1]:
    ;for
    std::println("ran out, holds ", $done);
    for $b: i8 [from: 126, to: 127, by: 1]:
        std::print($b, " ");
    ;for
    std::println.stored();
    let $words := ["a", "b", "c"];
    for perm $w in lend $words:
    ;for
    std::println.stored("last word ", $w);
;MAIN
