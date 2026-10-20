# A slice by grapheme or code point is text; by byte it is a list[u8], since a cut may land inside a
# character ("é" is 2 bytes), and text is made from it with to.str.utf8 (owner, 2026-10-20: "(a)").
MAIN {
    let 's' := "café!";
    let 'g' := str.slice.grapheme[lend 's', from: pos 2, to: pos 4] otherwise "?";
    let 'p' := str.slice.point[lend 's', from: pos 4, to: pos 5] otherwise "?";
    let 'b': list[u8] = str.slice.byte[lend 's', from: pos 4, to: pos 5] otherwise [];
    let 'half': list[u8] = str.slice.byte[lend 's', from: pos 4, to: pos 4] otherwise [];
    let 'back' := to.str.utf8['b'] otherwise "not utf-8";
    let 'cut' := to.str.utf8['half'] otherwise "not utf-8";
    let 'past' := str.slice.point[lend 's', from: pos 4, to: pos 9] otherwise "past the end";
    std::print.useful['g', " ", 'p', " ", 'b', " ", 'back', " ", 'cut', " ", 'past', newln];
}
