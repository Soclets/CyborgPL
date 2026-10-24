# Finding 2026-10-24 (generator, the writer episode; seeds 4, 16, 54, 99, 247 and more): kayo groups its prints
# into one buffer, which changes only its speed: the output order is unchanged (design claude, 2026-10-24;
# examples/print_writer.cy). Here "between" is printed before the writer is flushed, so it comes first, as
# in the oracle ("between", then "total: 7"). kayo writes the writer's "total: 7" first: its own buffer,
# still holding "between", goes out after the writer.
fn wrep [lent.mut $out: std::print.writer, $name: str.utf8, $n: i64] {
    std::print.useful(lend.mut $out, $name, ": ", $n, newln);
}

MAIN {
    let mut $pw := std::print.writer(size: 256 bytes);
    wrep(lend.mut $pw, "total", 7);
    std::print.useful("between", newln);
    std::print.flush(lend.mut $pw);
    std::print.useful("after", newln);
}
