proc report [lent.mut $out: std::print.writer, $name: str.utf8, $total: i64]:
    std::print(lend.mut $out, $name, ": ", $total, newln);
;proc report

MAIN:
    std::print("starting", newln);
    std::print.flush();

    var $out = std::print.writer(size: 4096 bytes);
    for $i: i64 [from: 1, to: 3, by: 1]:
        std::print(lend.mut $out, "line ", $i, newln);
    ;for
    report(lend.mut $out, "apples", 12);
    std::print("total: ", 7, newln)-
    otherwise:
        case write_failed:
            std::exit(code: 3);
        ;case write_failed
    ;otherwise;
;MAIN
