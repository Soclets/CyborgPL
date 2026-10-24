# Print (owner, 2026-10-23): kayo groups prints into one buffer and flushes it itself; `std::print.flush()` sends
# what it holds now, and a writer the program makes holds its own text, `std::print.writer(size: 4096 bytes)`, printed
# into when a print's first argument is `lend.mut $out`, and flushed with `std::print.flush(lend.mut $out)`. A writer
# that goes away still holding text is refused. A failed write stops the program, unless `otherwise` handles it.
fn report [lent.mut $out: std::print.writer, $name: str.utf8, $total: i64] {
    std::print.useful(lend.mut $out, $name, ": ", $total, newln);
}

MAIN {
    std::print.useful("starting", newln);
    std::print.flush();

    let mut $out := std::print.writer(size: 4096 bytes);
    for $i: i64 [from: 1, to: 3, by: 1] {
        std::print.useful(lend.mut $out, "line ", $i, newln);
    }
    report(lend.mut $out, "apples", 12);
    std::print.flush(lend.mut $out);
    std::print.useful("total: ", 7, newln) otherwise { write_failed { std::exit(code: 3); } };
}
