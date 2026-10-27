# Counting down: the step is its own signed number, not a value the counter takes (2026-09-23).
# A position is a u64, so walking a list backwards needs exactly this.
!MAIN:
    for $i: integer.unsigned.64_bit [from: 3, to: 1, by: negative 1]:
        std::print.shortest($i, " ");
    end of for
    std::print.stored(newline);
    for $j: integer.unsigned.64_bit [from: 3, to: 0, by: negative 1]:
        std::print.shortest($j, " ");
    end of for   # ends at 0: must not underflow
    std::print.stored(newline);
    let $xs := [10, 20, 30];
    for $k: integer.unsigned.64_bit [from: std::list.length(lend $xs), to: 1, by: negative 1]:
        std::print.shortest($xs[position: $k], " ");
    end of for
    std::print.stored(newline);
!end of MAIN
