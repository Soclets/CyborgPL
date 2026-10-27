# char conversions (decided 2026-09-21): code by name, text ↔ char and unit ↔ unit as failing conversions.
!MAIN:
    let $a := `A`;
    std::print.shortest(character.code($a), newline);                        # 65 — the number under it
    let $seven := to.integer.signed.32_bit(to.string.utf8(`7`)) otherwise negative 1;        # 7 — a digit's value is a text parse
    std::print.shortest($seven, newline);
    let $b := character.%code_point.from_code(66) otherwise `?`;
    let $bad := character.%code_point.from_code(1_114_112) otherwise `?`;    # past the last code point: not_a_char
    std::print.shortest($b, $bad, newline);                            # B?
    let $one := to.character.%code_point("Z") otherwise `?`;                 # text of exactly one point
    let $two := to.character.%code_point("ZZ") otherwise `?`;                # not_one_char
    std::print.shortest($one, $two, newline);                          # Z?
    let $g: character.%grapheme_cluster = `é`;
    let $p := to.character.%code_point($g) otherwise `?`;                   # this é is one point: fine
    let $y := to.character.byte($p) otherwise `?`;                    # é is two bytes: not_one_byte
    std::print.shortest($p, $y, newline);                              # é?
    let $up := to.character.%grapheme_cluster(`x`);                             # up never loses anything: no otherwise
    std::print.shortest(to.string.utf8($up) concatenate "!", newline);               # x!
!end of MAIN
