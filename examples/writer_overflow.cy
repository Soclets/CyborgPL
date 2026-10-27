# A print that does not fit a writer (owner, 2026-10-24): a writer, `.whole` written out or not, sends what it holds and
# keeps the print whole, one longer than the writer going out alone; `.split` fills to exactly its size, a print free
# to split, for raw throughput.
!MAIN:
    variable $pw := std::print.writer(size: 16 bytes);
    std::print.shortest(lend.mutable $pw, "w 1 32", newline);
    std::print.shortest(lend.mutable $pw, "w 2 64", newline);
    std::print.shortest(lend.mutable $pw, "total: 4527", newline);
    std::print.shortest("between", newline);
    std::print.flush(lend.mutable $pw);
    variable $sp := std::print.writer.split(size: 16 bytes);
    std::print.shortest(lend.mutable $sp, "w 1 32", newline);
    std::print.shortest(lend.mutable $sp, "w 2 64", newline);
    std::print.shortest(lend.mutable $sp, "total: 4527", newline);
    std::print.shortest("between", newline);
    std::print.flush(lend.mutable $sp);
    variable $wh := std::print.writer.whole(size: 8 bytes);
    std::print.shortest(lend.mutable $wh, "longer than eight", newline);
    std::print.flush(lend.mutable $wh);
!end of MAIN
