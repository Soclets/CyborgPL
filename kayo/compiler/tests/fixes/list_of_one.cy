# wrong_type: one bracketed value where a list is declared; a list of one is `[x,]` (design claude, 2026-10-25).
MAIN {
    let mut $mx: std::list[f32] = [0.0];
    std::list.append(lend.mut $mx, value: 1.0);
}
