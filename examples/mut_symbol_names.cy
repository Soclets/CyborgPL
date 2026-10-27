# A `mut` name kept changed through `lend.mut`, whatever it holds (check claude, 2026-10-16, corpus 65f59c7a:
# the oracle missed the change when the name held a `"` or a `\`).
procedure add [lent.mutable $a: std::list[integer.unsigned.16_bit]]:
    std::list.append(lend.mutable $a, value: 1);
end of procedure add
!MAIN:
    variable $'o"': std::list[integer.unsigned.16_bit] = [];
    add(lend.mutable $'o"');
    variable $'it\'s': std::list[integer.unsigned.16_bit] = [];
    add(lend.mutable $'it\'s');
    variable $'a\\b': std::list[integer.unsigned.16_bit] = [];
    add(lend.mutable $'a\\b');
    std::print.shortest(std::list.length(lend $'o"'), " ", std::list.length(lend $'it\'s'), " ", std::list.length(lend $'a\\b'), newline);
!end of MAIN
