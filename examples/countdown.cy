# Counting down: the step is its own signed number, not a value the counter takes (2026-09-23).
# A position is a u64, so walking a list backwards needs exactly this.
MAIN {
    for 'i': u64 [from: 3, to: 1, by: -1] { std::print.useful['i', " "]; }
    std::print.stored[newln];
    for 'j': u64 [from: 3, to: 0, by: -1] { std::print.useful['j', " "]; }   # ends at 0: must not underflow
    std::print.stored[newln];
    let 'xs' := [10, 20, 30];
    for 'k': u64 [from: list.length[lend 'xs'], to: 1, by: -1] { std::print.useful['xs'[pos: 'k'], " "]; }
    std::print.stored[newln];
}
