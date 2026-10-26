# What a chip with no OS holds without a heap (design claude, 2026-10-26):
# a fixed array in a variable, changed and printed, lives in the frame; a
# text literal in a variable stays where the program keeps it; a format
# hole is written into the runtime's one scratch buffer. And reaching MAIN's
# end calls the target's `exit` with 0 (owner, 2026-10-26: "yes").
MAIN {
    let mut $a: array[i32] = [5, 3, 9, 1,];
    change $a[pos: 2] = 7;
    std::print.useful($a[pos: 2], " ", $a, newln);
    let $t := "abc";
    std::print.useful($t, " ", 5, newln);
    let $x: u32 = 48879;
    std::print.format("{base: hex, zeros, width: 8 cols} {places: 2}|{width: 5 cols}|\n", $x, 2.5, true);
}
