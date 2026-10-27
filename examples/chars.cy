# char conversions (decided 2026-09-21): code by name, text ↔ char and unit ↔ unit as failing conversions.
!MAIN:
    let $a := `A`;
    std::print.shortest(char.code($a), newln);                        # 65 — the number under it
    let $seven := to.int.sign.32(to.str.utf8(`7`)) otherwise neg 1;        # 7 — a digit's value is a text parse
    std::print.shortest($seven, newln);
    let $b := char.%point.from_code(66) otherwise `?`;
    let $bad := char.%point.from_code(1_114_112) otherwise `?`;    # past the last code point: not_a_char
    std::print.shortest($b, $bad, newln);                            # B?
    let $one := to.char.%point("Z") otherwise `?`;                 # text of exactly one point
    let $two := to.char.%point("ZZ") otherwise `?`;                # not_one_char
    std::print.shortest($one, $two, newln);                          # Z?
    let $g: char.%cluster = `é`;
    let $p := to.char.%point($g) otherwise `?`;                   # this é is one point: fine
    let $y := to.char.byte($p) otherwise `?`;                    # é is two bytes: not_one_byte
    std::print.shortest($p, $y, newln);                              # é?
    let $up := to.char.%cluster(`x`);                             # up never loses anything: no otherwise
    std::print.shortest(to.str.utf8($up) concat "!", newln);               # x!
!end of MAIN
