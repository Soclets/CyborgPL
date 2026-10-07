# `format[…]` makes the text `print.format` prints (owner, 2026-10-07: "`a`"): one format language,
# to print or to make text. A print goes to stdout unless it says `.stderr` ("default to out"),
# and `.stdout` may still be written.
MAIN {
    let 'n' := 3;
    let 'line' := format["{} items at {:.2} each, 0x{:x}\n", 'n', 1.5, 255];
    print.useful['line'];
    print.weird.useful <- "hex: " <- format["{:x}", 'n' times 100] <- newln;
    print.useful.stdout["said ", "stdout", newln];
}
