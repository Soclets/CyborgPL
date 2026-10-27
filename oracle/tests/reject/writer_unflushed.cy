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
    std::print.shortest("total: ", 7, newline) otherwise:
        case write_failed:
            std::exit(code: 3);
        end of case write_failed
    end of otherwise;
!end of MAIN
