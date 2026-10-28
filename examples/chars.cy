# char conversions (decided 2026-09-21): code by name, text ↔ char and unit ↔ unit as failing conversions.
MAIN:
    let $a := `A`;
    std::print(char.code($a), newln);                        # 65 — the number under it
    let $seven := to.i32(to.str.utf8(`7`)) ?? -1;        # 7 — a digit's value is a text parse
    std::print($seven, newln);
    let $b := char.%point.from_code(66) ?? `?`;
    let $bad := char.%point.from_code(1_114_112) ?? `?`;    # past the last code point: not_a_char
    std::print($b, $bad, newln);                            # B?
    let $one := to.char.%point("Z") ?? `?`;                 # text of exactly one point
    let $two := to.char.%point("ZZ") ?? `?`;                # not_one_char
    std::print($one, $two, newln);                          # Z?
    let $g: char.%cluster := `é`;
    let $p := to.char.%point($g) ?? `?`;                   # this é is one point: fine
    let $y := to.char.byte($p) ?? `?`;                    # é is two bytes: not_one_byte
    std::print($p, $y, newln);                              # é?
    let $up := to.char.%cluster(`x`);                             # up never loses anything: no otherwise
    std::print(to.str.utf8($up) ++ "!", newln);               # x!
;MAIN
