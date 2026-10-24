# Finding 2026-10-24 (generator, the writer episode with a 16-byte writer; seeds 54, 187, 238, 290): a print
# that does not fit in what a writer has left. The writer holds "w 1 32\n" and "w 2 64\n" (14 bytes); then
# "total: 4527\n" (12) overflows it. The oracle sends the writer's text with the whole of that print
# before "between"; kayo fills the writer to its 16 bytes ("…to"), sends those, prints "between", and
# sends "tal: 4527" at the flush, so the line is split by another print. Which is meant is not written
# (design/print notes: a writer's text goes out "at its flush or when it fills"); asked of design claude.
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
