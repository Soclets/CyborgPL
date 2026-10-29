# wrong_type: one bracketed value where a list is declared; a list of one is `[x,]` (design claude, 2026-10-25).
MAIN:
    var $mx: std::ls[f32] = [0.0];
    std::ls.append(lend.mut $mx, value: 1.0);
;MAIN
