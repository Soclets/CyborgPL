# A slice by grapheme or code point is text; by byte it is a list[u8], since a cut may land inside a
# character ("é" is 2 bytes), and text is made from it with to.str.utf8 (owner, 2026-10-20: "(a)").
!MAIN:
    let $s := "café!";
    let $g := std::str.slice.cluster(lend $s, from: pos 2, to: pos 4) otherwise "?";
    let $p := std::str.slice.point(lend $s, from: pos 4, to: pos 5) otherwise "?";
    let $b: std::ls[int.unsign.8] := std::str.slice.byte(lend $s, from: pos 4, to: pos 5) otherwise [];
    let $half: std::ls[int.unsign.8] := std::str.slice.byte(lend $s, from: pos 4, to: pos 4) otherwise [];
    let $back := to.str.utf8($b) otherwise "not utf-8";
    let $cut := to.str.utf8($half) otherwise "not utf-8";
    let $past := std::str.slice.point(lend $s, from: pos 4, to: pos 9) otherwise "past the end";
    std::print.shortest($g, " ", $p, " ", $b, " ", $back, " ", $cut, " ", $past, newln);
!end of MAIN
