# Finding 2026-10-22 (by hand, adding operator paths to the generator): a kit's operators, glyphs
# included, take an optional path (owner, 2026-10-22), and `xx` is the symbol of std's `pow`
# (2026-10-16), so `'a' std::xx 2` is `'a' pow 2` (design claude's reading; oracle edcbf2cb,
# examples/std_glyph_path.cy). kayo refuses it: "no kit with the prefix `std`".
MAIN {
    let 'a': i64 = 7;
    std::print.useful('a' std::xx 2, " ", 'a' xx 2, " ", 'a' std::pow 2, newln);
}
