# char conversions (decided 2026-09-21): code by name, text ↔ char and unit ↔ unit as failing conversions.
MAIN:
    let $a := `A`;
    std::println(char.code($a));                        # 65 — the number under it
    let $seven := to.i32(to.str.utf8(`7`)) ?? -1;        # 7 — a digit's value is a text parse
    std::println($seven);
    let $b := char.%point.from_code(66) ?? `?`;
    let $bad := char.%point.from_code(1_114_112) ?? `?`;    # past the last code point: not_a_char
    std::println($b, $bad);                            # B?
    let $one := to.char.%point("Z") ?? `?`;                 # text of exactly one point
    let $two := to.char.%point("ZZ") ?? `?`;                # not_one_char
    std::println($one, $two);                          # Z?
    let $g: char.%cluster := `é`;
    let $p := to.char.%point($g) ?? `?`;                   # this é is one point: fine
    let $y := to.char.byte($p) ?? `?`;                    # é is two bytes: not_one_byte
    std::println($p, $y);                              # é?
    let $up := to.char.%cluster(`x`);                             # up never loses anything: no otherwise
    std::println(to.str.utf8($up) ++ "!");               # x!
;MAIN
