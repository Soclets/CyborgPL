# A print that does not fit a writer (owner, 2026-10-24): a writer, `.whole` written out or not, sends what it holds and
# keeps the print whole, one longer than the writer going out alone; `.split` fills to exactly its size, a print free
# to split, for raw throughput.
MAIN:
    var $pw = std::print.writer(size: 16 bytes);
    std::println(lend.mut $pw, "w 1 32");
    std::println(lend.mut $pw, "w 2 64");
    std::println(lend.mut $pw, "total: 4527");
    std::println("between");
    std::print.flush(lend.mut $pw);
    var $sp = std::print.writer.split(size: 16 bytes);
    std::println(lend.mut $sp, "w 1 32");
    std::println(lend.mut $sp, "w 2 64");
    std::println(lend.mut $sp, "total: 4527");
    std::println("between");
    std::print.flush(lend.mut $sp);
    var $wh = std::print.writer.whole(size: 8 bytes);
    std::println(lend.mut $wh, "longer than eight");
    std::print.flush(lend.mut $wh);
;MAIN
