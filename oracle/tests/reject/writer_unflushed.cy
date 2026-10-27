proc report [lent.mut $out: std::print.writer, let $name: str.utf8, let $total: int.sign.64]:
    std::print.shortest(lend.mut $out, $name, ": ", $total, newln);
end of proc report

!MAIN:
    std::print.shortest("starting", newln);
    std::print.flush();

    var $out = std::print.writer(size: 4096 bytes);
    for $i: int.sign.64 [from: 1, to: 3, by: 1]:
        std::print.shortest(lend.mut $out, "line ", $i, newln);
    end of for
    report(lend.mut $out, "apples", 12);
    std::print.shortest("total: ", 7, newln)-
    otherwise:
        case write_failed:
            std::exit(code: 3);
        end of case write_failed
    end of otherwise;
!end of MAIN
