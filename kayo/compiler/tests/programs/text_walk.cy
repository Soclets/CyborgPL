# Walking a text by its units (strings.md): str.each.byte / .point /
# .grapheme — 21 bytes, 8 points, 6 clusters for "aé" ++ "เกี่ยว"; each
# cluster a text of its own, given back with the walk.
%extern.C proc exit [$code: i32];
%extern.C proc getenv [$name: ptr[u8]] -> ptr[u8];
MAIN:
    let $k := "SYNAPSE_SURELY_UNSET_VARIABLE";
    no_check let $there := ptr.is_null(getenv(lend $k));
    var $s = "ab";
    if $there:
        change $s to "aé" concat "เกี่ยว";
    ;if
    var $bytes: i32 = 0;
    for $b in std::str.each.byte(lend $s):
        change $bytes to $bytes + 1;
    ;for
    var $points: i32 = 0;
    var $codes: i64 = 0;
    for $p in std::str.each.point(lend $s):
        change $points to $points + 1; change $codes to $codes + to.i64(char.code($p));
    ;for
    var $clusters: i32 = 0;
    var $last = str.empty;
    for $g in std::str.each.cluster(lend $s):
        change $clusters to $clusters + 1; change $last to to.str.utf8(lend $g);
    ;for
    var $got: i32 = 0;
    if $bytes == 21 and $points == 8:
        change $got to $got + 1;
    ;if
    if $codes == 97 + 233 + 3648 + 3585 + 3637 + 3656 + 3618 + 3623:
        change $got to $got + 2;
    ;if
    if $clusters == 6 and $last == "ว":
        change $got to $got + 4;
    ;if
    no_check my.exit($got);
;MAIN
