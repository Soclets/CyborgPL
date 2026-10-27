# Print (owner, 2026-10-23): kayo groups prints into one buffer and flushes it itself; `std::print.flush()` sends
# what it holds now, and a writer the program makes holds its own text, `std::print.writer(size: 4096 bytes)`, printed
# into when a print's first argument is `lend.mut $out`, and flushed with `std::print.flush(lend.mut $out)`. A writer
# that goes away still holding text is refused. A failed write stops the program, unless `otherwise` handles it.
proc report [lent.mut $out: std::print.writer, $name: str.utf8, $total: int.sign.64]:
    std::print.shortest(lend.mut $out, $name, ": ", $total, newln);
end of proc report

!MAIN:
    std::print.shortest("starting", newln);
    std::print.flush();

    var $out := std::print.writer(size: 4096 bytes);
    for $i: int.sign.64 [from: 1, to: 3, by: 1]:
        std::print.shortest(lend.mut $out, "line ", $i, newln);
    end of for
    report(lend.mut $out, "apples", 12);
    std::print.flush(lend.mut $out);
    std::print.shortest("total: ", 7, newln)-
    otherwise:
        case write_failed:
            std::exit(code: 3);
        end of case write_failed
    end of otherwise;
!end of MAIN
