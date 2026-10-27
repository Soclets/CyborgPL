# `format(…)` makes the text `print.format` prints (owner, 2026-10-07: "`a`"): one format language,
# to print or to make text. A print goes to stdout unless it says `.stderr` ("default to out"),
# and `.stdout` may still be written.
!MAIN:
    let $n := 3;
    let $line := std::format("{} items at {places: 2} each, 0x{base: hexadecimal}{newline}", $n, 1.5, 255);
    std::print.shortest($line);
    std::print.arrows.shortest <- "hex: " <- std::format("{base: hexadecimal}", $n times 100) <- newline;
    std::print.shortest.stdout("said ", "stdout", newline);
!end of MAIN
