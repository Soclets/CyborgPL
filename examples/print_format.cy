# std::print.format — the formatting print (decided 2026-09-22; `{}` holes since 2026-10-01).
# `{}` takes the next value in its readable form; `{1}`, `{2}`… a numbered one (counted from 1, never
# mixed with `{}`); `:` adds a shape: fill and alignment `< ^ >`, `+`, `0` padding, a width, `.places`,
# and a kind — times X o b (bases), e (scientific), ? (the stored form). The format text has its own
# escapes, read in one pass: newln \t \\ \" \{ \} and the rest. `%` is plain text.
!MAIN:
    let $name := "rope";
    let $count := 3;
    let $price: float.binary.64_bit = 12.5;
    let $pay: float.%decimal.64_bit = 37.5 times 12.40;
    let $tenth: float.binary.64_bit = 0.1;
    std::print.format("{}: {} at ${places: 2}, 50% off{newline}", $name, $count, $price);
    std::print.format("pay ${places: 2}, readable {}, stored {form: stored}{newline}", $pay, $pay, $pay);
    std::print.format("tenth {} is really {form: stored}{newline}", $tenth, $tenth);
    std::print.format("0x{base: hexadecimal} 0x{base: hexadecimal.upper, zeros, width: 4 columns} 0o{base: octal} 0b{base: binary, zeros, width: 8 columns}{newline}", 255, 255, 8, 10);
    std::print.format("[{align: right, width: 6 columns}] [{align: left, width: 6 columns}] [{align: center, width: 7 columns, fill: `*`}] [{sign}] [{zeros, width: 5 columns}] {form: science, places: 2}{newline}", 42, 42, "ab", 5, 42, $price);
    std::print.format("{2} vs {1}, and {2} wins{newline}", "a", "b");
    std::print.format("\{ \"count\": {} \} in C:\\temp, tab[{tab}]{newline}", $count);
!end of MAIN
