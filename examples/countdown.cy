# Counting down: the step is its own signed number, not a value the counter takes (2026-09-23).
# A position is a u64, so walking a list backwards needs exactly this.
!MAIN:
    for $i: int.unsign.64 [from: 3, to: 1, by: neg 1]:
        std::print.shortest($i, " ");
    end of for
    std::print.stored(newln);
    for $j: int.unsign.64 [from: 3, to: 0, by: neg 1]:
        std::print.shortest($j, " ");
    end of for   # ends at 0: must not underflow
    std::print.stored(newln);
    let $xs := [10, 20, 30];
    for $k: int.unsign.64 [from: std::ls.len(lend $xs), to: 1, by: neg 1]:
        std::print.shortest($xs[pos: $k], " ");
    end of for
    std::print.stored(newln);
!end of MAIN
