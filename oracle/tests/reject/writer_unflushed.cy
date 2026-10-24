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
    std::print.useful("total: ", 7, newln) otherwise { write_failed { std::exit(code: 3); } };
}
