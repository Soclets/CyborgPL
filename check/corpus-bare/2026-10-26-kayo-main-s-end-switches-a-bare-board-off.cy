# Ruled 2026-10-26 (owner: "yes"; DECISIONS): on a chip with no operating system, reaching MAIN's end calls the
# target's `exit` with 0 when the target names one (here `my.board.off`); with none named the CPU sleeps. This MAIN
# has no `std::exit`: it prints and ends, so the board must switch off with the two lines out, as the oracle prints.
# kayo calls it since 78a41606.
MAIN {
    std::print.useful("first", newln);
    std::print.useful(6 x 7, newln);
}
