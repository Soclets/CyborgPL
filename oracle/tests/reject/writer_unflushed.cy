proc report [lent.mut $out: std::print.writer, let $name: str.utf8, let $total: i64]:
    std::println(lend.mut $out, $name, ": ", $total);
;proc report

MAIN:
    std::println("starting");
    std::print.flush();

    var $out = std::print.writer(size: 4096 bytes);
    for $i: i64 [from: 1, to: 3, by: 1]:
        std::println(lend.mut $out, "line ", $i);
    ;for
    report(lend.mut $out, "apples", 12);
    std::println("total: ", 7)-
    otherwise:
        case write_failed:
            std::exit(code: 3);
        ;case write_failed
    ;otherwise;
;MAIN
