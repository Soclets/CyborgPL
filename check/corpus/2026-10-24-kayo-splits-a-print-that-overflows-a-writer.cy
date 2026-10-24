# Finding 2026-10-24 (generator, the writer episode with a 16-byte writer; seeds 54, 187, 238, 290): a print
# that does not fit in what a writer has left. Ruled (owner, 2026-10-24; oracle deda6720,
# examples/writer_overflow.cy): a writer sends what it holds and keeps the new print whole, so "between"
# comes first and "total: 4527" after it, in one piece; only `std::print.writer.split(…)` fills to exactly
# its size. kayo still fills a default writer to its 16 bytes ("…to"), prints "between", and sends
# "tal: 4527" at the flush.
fn wrep [lent.mut $out: std::print.writer, $name: str.utf8, $n: i64] {
    std::print.useful(lend.mut $out, $name, ": ", $n, newln);
}

MAIN {
    let mut $pw := std::print.writer(size: 16 bytes);
    std::print.useful(lend.mut $pw, "w 1 32", newln);
    std::print.useful(lend.mut $pw, "w 2 64", newln);
    wrep(lend.mut $pw, "total", 4527);
    std::print.useful("between", newln);
    std::print.flush(lend.mut $pw);
}
