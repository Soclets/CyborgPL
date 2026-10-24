# A print that does not fit a writer (owner, 2026-10-24): a writer, `.whole` written out or not, sends what it holds and
# keeps the print whole, one longer than the writer going out alone; `.split` fills to exactly its size, a print free
# to split, for raw throughput.
MAIN {
    let mut $pw := std::print.writer(size: 16 bytes);
    std::print.useful(lend.mut $pw, "w 1 32", newln);
    std::print.useful(lend.mut $pw, "w 2 64", newln);
    std::print.useful(lend.mut $pw, "total: 4527", newln);
    std::print.useful("between", newln);
    std::print.flush(lend.mut $pw);
    let mut $sp := std::print.writer.split(size: 16 bytes);
    std::print.useful(lend.mut $sp, "w 1 32", newln);
    std::print.useful(lend.mut $sp, "w 2 64", newln);
    std::print.useful(lend.mut $sp, "total: 4527", newln);
    std::print.useful("between", newln);
    std::print.flush(lend.mut $sp);
    let mut $wh := std::print.writer.whole(size: 8 bytes);
    std::print.useful(lend.mut $wh, "longer than eight", newln);
    std::print.flush(lend.mut $wh);
}
