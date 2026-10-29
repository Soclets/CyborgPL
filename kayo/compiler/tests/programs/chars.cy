# Characters (types.md, conversion.md): a point is its code, 32 bits; a
# byte 8. The code by name, from a code failing not_a_char (a point is a
# Unicode scalar, a byte 0-255), text <-> char and point -> byte as
# failing conversions, a char as text (a byte as the point of its number,
# the oracle's reading), `==` across units, and `++` beside text. Values
# are known only at run time: whether an unset variable is there.
%extern.C proc exit [$code: i32];
%extern.C proc getenv [$name: ptr[u8]] -> ptr[u8];
fn point [$code: i64] -> u32:
    let $c := char.%point.from_code($code)-
    otherwise:
        case not_a_char:
            ret 0;
        ;case not_a_char
    ;otherwise;
    ret char.code($c);
;fn point
fn byte [$code: i64] -> u8:
    let $c := char.byte.from_code($code)-
    otherwise:
        case not_a_char:
            ret 0;
        ;case not_a_char
    ;otherwise;
    ret char.code($c);
;fn byte
MAIN:
    let $k := "SYNAPSE_SURELY_UNSET_VARIABLE";
    no_check let $there := ptr.is_null(getenv(lend $k));
    var $d: i64 = 0;
    if $there:
        change $d to 1;
    ;if
    var $got: i32 = 0;
    if point(55295 + $d) == 0 and point(57344 - $d) == 0 and point(-$d) == 0 and point(1114110 + $d) == 1114111:
        change $got to $got + 1;
    ;if
    if byte(199 + $d) == 200 and byte(255 + $d) == 0 and point($d - 1) == 0:
        change $got to $got + 2;
    ;if
    let $b200 := char.byte.from_code(199 + $d) ?? `?`;
    let $t := to.str.utf8($b200);
    if std::str.bytes(lend $t) == 2:
        change $got to $got + 4;
    ;if
    var $e = "Z";
    if $there:
        change $e to "é";
    ;if
    let $p := to.char.%point(lend $e) ?? `?`;
    let $y := to.char.byte($p) ?? `?`;
    if char.code($p) == 233 and $y == `?`:
        change $got to $got + 8;
    ;if
    let $two := to.char.%point($e concat "x") ?? `?`;
    let $none := to.char.byte(str.empty concat $e) ?? `!`;
    if $two == `?` and $none == `!`:
        change $got to $got + 16;
    ;if
    let $bb: char.byte := `A`;
    var $pp: char.%point = `B`;
    if $there:
        change $pp to `A`;
    ;if
    if $bb == $pp:
        change $got to $got + 32;
    ;if
    let $j := $p concat "!" concat $pp;
    if $j == "é!A":
        change $got to $got + 64;
    ;if
    no_check my.exit($got);
;MAIN
