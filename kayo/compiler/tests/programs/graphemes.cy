# Graphemes (types.md): a char.grapheme is a text of its own; counts in
# points and grapheme clusters (UAX #29, extended: เกี่ยว is 4 clusters, 6
# points, 18 bytes); text <-> grapheme conversions; a text bound counted
# in graphemes, checked at run time. Values chosen at run time.
%extern.C proc exit [$code: i32];
%extern.C proc getenv [$name: ptr[u8]] -> ptr[u8];
fn short [lent $t: str.utf8] -> i32:
    let $s: str.utf8[min: 0, max: 2 %clusters] := to.str.utf8[min: 0, max: 2 %clusters](lend $t)-
    otherwise:
        case out_of_range:
            ret 0;
        ;case out_of_range
    ;otherwise;
    ret 1;
;fn short
MAIN:
    let $k := "SYNAPSE_SURELY_UNSET_VARIABLE";
    no_check let $there := ptr.is_null(getenv(lend $k));
    var $w = "ab";
    if $there:
        change $w to "เกี่ยว";
    ;if
    var $got: i32 = 0;
    if std::str.clusters(lend $w) == 4 and std::str.points(lend $w) == 6 and std::str.bytes(lend $w) == 18:
        change $got to $got + 1;
    ;if
    let $g: char.%cluster := `é`;
    let $p := to.char.%point(lend $g) ?? `?`;
    if char.code($p) == 233 and $g == `é`:
        change $got to $got + 2;
    ;if
    let $one := to.char.%cluster("x" concat $w) ?? `?`;
    let $syl := to.char.%cluster("เกี่ยว") ?? `?`;
    let $acc := to.char.%cluster("é") ?? `?`;
    let $at := to.str.utf8(lend $acc);
    if $one == `?` and $syl == `?` and std::str.bytes(lend $at) == 3:
        change $got to $got + 4;
    ;if
    let $up := to.char.%cluster(`x`);
    if to.str.utf8(lend $up) concat "!" == "x!":
        change $got to $got + 8;
    ;if
    if short(lend $w) == 0 and short("ab") == 1 and short("abc") == 0:
        change $got to $got + 16;
    ;if
    no_check my.exit($got);
;MAIN
