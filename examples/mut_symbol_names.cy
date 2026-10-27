# A `mut` name kept changed through `lend.mut`, whatever it holds (check claude, 2026-10-16, corpus 65f59c7a:
# the oracle missed the change when the name held a `"` or a `\`).
proc add [lent.mut $a: std::ls[int.unsign.16]]:
    std::ls.append(lend.mut $a, value: 1);
end of proc add
!MAIN:
    var $'o"': std::ls[int.unsign.16] := [];
    add(lend.mut $'o"');
    var $'it\'s': std::ls[int.unsign.16] := [];
    add(lend.mut $'it\'s');
    var $'a\\b': std::ls[int.unsign.16] := [];
    add(lend.mut $'a\\b');
    std::print.shortest(std::ls.len(lend $'o"'), " ", std::ls.len(lend $'it\'s'), " ", std::ls.len(lend $'a\\b'), newln);
!end of MAIN
