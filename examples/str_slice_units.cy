# A slice by grapheme or code point is text; by byte it is a list[u8], since a cut may land inside a
# character ("é" is 2 bytes), and text is made from it with to.str.utf8 (owner, 2026-10-20: "(a)").
!MAIN:
    let $s := "café!";
    let $g := std::string.slice.grapheme_cluster(lend $s, from: position 2, to: position 4) otherwise "?";
    let $p := std::string.slice.code_point(lend $s, from: position 4, to: position 5) otherwise "?";
    let $b: std::list[integer.unsigned.8_bit] = std::string.slice.byte(lend $s, from: position 4, to: position 5) otherwise [];
    let $half: std::list[integer.unsigned.8_bit] = std::string.slice.byte(lend $s, from: position 4, to: position 4) otherwise [];
    let $back := to.string.utf8($b) otherwise "not utf-8";
    let $cut := to.string.utf8($half) otherwise "not utf-8";
    let $past := std::string.slice.code_point(lend $s, from: position 4, to: position 9) otherwise "past the end";
    std::print.shortest($g, " ", $p, " ", $b, " ", $back, " ", $cut, " ", $past, newline);
!end of MAIN
