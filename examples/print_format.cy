# std::print.format — the formatting print (decided 2026-09-22; `{}` holes since 2026-10-01).
# `{}` takes the next value in its readable form; `{1}`, `{2}`… a numbered one (counted from 1, never
# mixed with `{}`); `:` adds a shape: fill and alignment `< ^ >`, `+`, `0` padding, a width, `.places`,
# and a kind — times X o b (bases), e (scientific), ? (the stored form). The format text has its own
# escapes, read in one pass: newln \t \\ \" \{ \} and the rest. `%` is plain text.
MAIN {
    let 'name' := "rope";
    let 'count' := 3;
    let 'price': f64 = 12.5;
    let 'pay': d64 = 37.5 x 12.40;
    let 'tenth': f64 = 0.1;
    std::print.format["{}: {} at ${:.2}, 50% off\n", 'name', 'count', 'price'];
    std::print.format["pay ${:.2}, readable {}, stored {:?}\n", 'pay', 'pay', 'pay'];
    std::print.format["tenth {} is really {:?}\n", 'tenth', 'tenth'];
    std::print.format["0x{:x} 0x{:04X} 0o{:o} 0b{:08b}\n", 255, 255, 8, 10];
    std::print.format["[{:>6}] [{:<6}] [{:*^7}] [{:+}] [{:05}] {:.2e}\n", 42, 42, "ab", 5, 42, 'price'];
    std::print.format["{2} vs {1}, and {2} wins\n", "a", "b"];
    std::print.format["\{ \"count\": {} \} in C:\\temp, tab[\t]\n", 'count'];
}
