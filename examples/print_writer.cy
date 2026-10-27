# Print (owner, 2026-10-23): kayo groups prints into one buffer and flushes it itself; `std::print.flush()` sends
# what it holds now, and a writer the program makes holds its own text, `std::print.writer(size: 4096 bytes)`, printed
# into when a print's first argument is `lend.mut $out`, and flushed with `std::print.flush(lend.mut $out)`. A writer
# that goes away still holding text is refused. A failed write stops the program, unless `otherwise` handles it.
procedure report [lent.mutable $out: std::print.writer, $name: string.utf8, $total: integer.signed.64_bit]:
    std::print.shortest(lend.mutable $out, $name, ": ", $total, newline);
end of procedure report

!MAIN:
    std::print.shortest("starting", newline);
    std::print.flush();

    variable $out := std::print.writer(size: 4096 bytes);
    for $i: integer.signed.64_bit [from: 1, to: 3, by: 1]:
        std::print.shortest(lend.mutable $out, "line ", $i, newline);
    end of for
    report(lend.mutable $out, "apples", 12);
    std::print.flush(lend.mutable $out);
    std::print.shortest("total: ", 7, newline) otherwise:
        case write_failed:
            std::exit(code: 3);
        end of case write_failed
    end of otherwise;
!end of MAIN
